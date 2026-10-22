#!/usr/bin/env python3
"""Generate RN rule catalog (JSON, Rust, TS) and test case skeleton from rn_layout_*.csv."""
from __future__ import annotations

import csv
import json
import re
from collections import defaultdict
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
NFSE_CSV = ROOT / "rn_layout_dps_nfse.csv"
RECEP_CSV = ROOT / "rn_layout_recepcao_dps.csv"
OVERRIDES = ROOT / "rn_rule_overrides.yaml"

OUT_JSON = ROOT / "backend/app/services/nfse/dps_rn_catalog.json"
OUT_RUST = ROOT / "backend/app/services/nfse/dps_rn_catalog.rs"
OUT_TS = ROOT / "frontend/src/nfse/rnCatalog.generated.ts"
OUT_CASES = ROOT / "backend/app/services/nfse/rn/rn_cases.json"
OUT_CASES_FE = ROOT / "frontend/src/nfse/fixtures/rn_cases.json"
OUT_DISPATCH = ROOT / "backend/app/services/nfse/rn/dispatch_generated.rs"


def load_obrig_rej(path: Path) -> list[dict]:
    rows: list[dict] = []
    with path.open(newline="", encoding="utf-8") as f:
        for rec in csv.reader(f):
            if len(rec) < 8:
                continue
            aplic, efeito, cod = rec[5].strip(), rec[6].strip(), rec[7].strip()
            if "obrig" not in aplic.lower() or not efeito.lower().startswith("rej"):
                continue
            if not cod:
                continue
            rows.append(
                {
                    "cod_erro": cod,
                    "xml_path": rec[1].strip(),
                    "campo": rec[2].strip(),
                    "descricao": rec[3].strip(),
                    "msg_erro": rec[8].strip() if len(rec) > 8 else "",
                    "recv_dps": rec[10].strip().upper() if len(rec) > 10 else "",
                    "nivel_regra": rec[9].strip() if len(rec) > 9 else "",
                }
            )
    return rows


def infer_group(r: dict) -> str:
    p, c = r["xml_path"].lower(), r["campo"].lower()
    d = r["descricao"].lower()
    if "ibscbs" in p or c == "ibscbs":
        return "Ibscbs"
    if "/toma/" in p or ("tomador" in d and "prestador" not in d[:20]):
        return "Tomador"
    if "/prest/" in p or ("prestador" in d and "tomador" not in d):
        return "Prestador"
    if "/interm/" in p:
        return "Intermediario"
    if "subst" in p:
        return "Substituicao"
    if "comext" in p or "exportação" in d or "exportacao" in d:
        return "Exportacao"
    if "atvevento" in p:
        return "Evento"
    if "vdedred" in p.replace(" ", ""):
        return "DeducaoReducao"
    if c in ("vserv", "vbc", "vliq") or "valor" in d:
        return "Valores"
    if "issqn" in p or c in ("tribissqn", "tpretissqn", "regesptrib", "tpsusp", "nprocesso"):
        return "Issqn"
    if c in ("cst", "tpretpiscofins") or "pis" in d or "cofins" in d:
        return "PisCofins"
    if c in ("cep", "xlgr", "nro", "xbairro", "end", "endnac", "endext"):
        return "Endereco"
    if "infdps" in p.replace(" ", "").lower() or "/dps/" in p.lower():
        if c in ("tpamb", "serie", "ndps", "dcompet", "dhemi", "id", "tpemit", "finnfse"):
            return "IdentificacaoDps"
        if c in ("ctribnac", "ctribmun", "xdescserv", "cintcontrib"):
            return "ServicoItem"
        return "IdentificacaoDps"
    if p.startswith("NFSe/") and "infDPS" not in p and "DPS" not in p:
        return "NfsePosProcessamento"
    if c == "versao":
        return "Plataforma"
    if c == "signature":
        return "Assinatura"
    if c in ("cmun", "cpais", "nif", "im", "email", "cnpj", "cpf", "xnome"):
        return "CadastroParte"
    if "catalogo" in d or "parametriz" in d or "e0312" in d.lower():
        return "CatalogoMunicipal"
    return "RegraGeral"


def infer_validator_id(r: dict, group: str) -> str:
    cod = r["cod_erro"]
    c = r["campo"].lower()
    overrides = load_overrides()
    if cod in overrides and "validator_id" in overrides[cod]:
        return overrides[cod]["validator_id"]
    if group == "Tomador" and cod == "E0202":
        return "tomador.prestador_diferente"
    if group == "Plataforma":
        return "plataforma.layout"
    if group == "CatalogoMunicipal":
        return "catalogo.adn"
    if c:
        return f"{group.lower()}.{c}"
    return f"{group.lower()}.{cod.lower()}"


def infer_form_fields(r: dict, group: str) -> list[str]:
    c = r["campo"]
    mapping = {
        "tpAmb": ["ambiente"],
        "serie": ["serie"],
        "nDPS": ["numero_documento"],
        "dCompet": ["data_competencia"],
        "vServ": ["valor_total_servicos_produtos"],
        "cTribNac": ["itens.codigo_tributacao_nacional"],
        "cTribMun": ["itens.codigo_tributacao_municipal"],
        "CNPJ": ["cliente_id", "empresa_id"],
        "CPF": ["cliente_id"],
        "tribISSQN": ["trib_issqn"],
        "CST": ["cst_pis_cofins"],
        "tpRetPisCofins": ["tp_ret_pis_cofins_csll"],
    }
    if c in mapping:
        return mapping[c]
    if group == "Tomador":
        return ["cliente_id", "tomador_localidade"]
    if group == "Prestador":
        return ["empresa_id"]
    if group == "ServicoItem":
        return ["itens"]
    return []


def load_overrides() -> dict:
    if not OVERRIDES.is_file():
        return {}
    try:
        import yaml  # type: ignore
    except ImportError:
        return {}
    with OVERRIDES.open(encoding="utf-8") as f:
        data = yaml.safe_load(f) or {}
    return data.get("rules", data) if isinstance(data, dict) else {}


def merge_rules(nfse: list[dict], recep: list[dict]) -> list[dict]:
    by_cod: dict[str, dict] = {}
    for r in nfse:
        by_cod[r["cod_erro"]] = {**r, "fonte_nfse": True, "fonte_recepcao": False}
    for r in recep:
        if r["cod_erro"] in by_cod:
            by_cod[r["cod_erro"]]["fonte_recepcao"] = True
            if r["recv_dps"]:
                by_cod[r["cod_erro"]]["recv_dps"] = r["recv_dps"]
        else:
            by_cod[r["cod_erro"]] = {**r, "fonte_nfse": False, "fonte_recepcao": True}
    catalog = []
    for cod in sorted(by_cod.keys()):
        r = by_cod[cod]
        group = infer_group(r)
        overrides = load_overrides()
        o = overrides.get(cod, {})
        group = o.get("group", group)
        entry = {
            "cod_erro": cod,
            "msg_erro": r["msg_erro"],
            "xml_path": r["xml_path"],
            "campo": r["campo"],
            "descricao": r["descricao"],
            "group": group,
            "validator_id": o.get("validator_id", infer_validator_id(r, group)),
            "form_fields": o.get("form_fields", infer_form_fields(r, group)),
            "requires_db_catalog": group == "CatalogoMunicipal"
            or cod in ("E0312", "E0314", "E0315", "E0539"),
            "recv_dps": r.get("recv_dps", ""),
            "fonte_nfse": r.get("fonte_nfse", False),
            "fonte_recepcao": r.get("fonte_recepcao", False),
        }
        catalog.append(entry)
    return catalog


def rust_group_enum(groups: set[str]) -> str:
    lines = [
        "#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]",
        "pub enum RuleGroup {",
    ]
    for g in sorted(groups):
        lines.append(f"    {g},")
    lines.append("}")
    return "\n".join(lines)


def emit_rust(catalog: list[dict]) -> str:
    groups = {c["group"] for c in catalog}
    group_enum = rust_group_enum(groups)
    rules_json = json.dumps(catalog, ensure_ascii=False, indent=2)
    return f"""//! Generated by scripts/gen_rn_catalog.py — do not edit.
#![allow(clippy::all)]

use serde::{{Deserialize, Serialize}};

{group_enum}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RnRuleMeta {{
    pub cod_erro: &'static str,
    pub msg_erro: &'static str,
    pub xml_path: &'static str,
    pub campo: &'static str,
    pub group: RuleGroup,
    pub validator_id: &'static str,
    pub requires_db_catalog: bool,
}}

/// All Obrig.+Rej. rules (deduped by cod_erro).
pub static ALL_RULES: &[RnRuleMeta] = &[
{"".join(
    f'    RnRuleMeta {{ cod_erro: "{c["cod_erro"]}", msg_erro: {json.dumps(c["msg_erro"], ensure_ascii=False)}, xml_path: {json.dumps(c["xml_path"], ensure_ascii=False)}, campo: {json.dumps(c["campo"], ensure_ascii=False)}, group: RuleGroup::{c["group"]}, validator_id: {json.dumps(c["validator_id"], ensure_ascii=False)}, requires_db_catalog: {"true" if c["requires_db_catalog"] else "false"} }},\n'
    for c in catalog
)}
];

pub fn rule_by_cod(cod: &str) -> Option<&'static RnRuleMeta> {{
    ALL_RULES.iter().find(|r| r.cod_erro == cod)
}}

pub fn catalog_json_embedded() -> &'static str {{
    include_str!("dps_rn_catalog.json")
}}
"""


def emit_ts(catalog: list[dict]) -> str:
    return f"""/** Generated by scripts/gen_rn_catalog.py — do not edit. */
export type RuleGroup =
{chr(10).join(f"  | '{g}'" for g in sorted({c['group'] for c in catalog}))};

export interface RnRuleMeta {{
  cod_erro: string;
  msg_erro: string;
  xml_path: string;
  campo: string;
  group: RuleGroup;
  validator_id: string;
  form_fields: string[];
  requires_db_catalog: boolean;
  recv_dps: string;
}}

export const ALL_RULES: readonly RnRuleMeta[] = {json.dumps(catalog, ensure_ascii=False, indent=2)} as const;

export function ruleByCod(cod: string): RnRuleMeta | undefined {{
  return ALL_RULES.find((r) => r.cod_erro === cod);
}}
"""


def default_mutate(c: dict) -> tuple[dict, bool]:
    """Return (mutate, pending) heuristics for test cases."""
    cod, group, campo = c["cod_erro"], c["group"], c["campo"].lower()
    if group in ("Assinatura",):
        return {}, False
    if group == "Ibscbs":
        return {}, False
    if group in ("DeducaoReducao", "Exportacao", "Evento", "Substituicao"):
        return {}, False
    if cod == "E0202":
        return {"tomador_same_as_prestador": True}, False
    if campo == "vserv" or cod in ("E0001", "E0004"):
        return {"valor_total_servicos_produtos": "0"}, False
    if campo == "ctribnac" and cod != "E0312":
        return {"clear_c_trib_nac": True}, False
    if cod == "E0312":
        return {}, False
    if cod == "E0315":
        return {"codigo_tributacao_municipal": "000"}, False
    if cod.startswith("E031") or group == "CatalogoMunicipal":
        return {}, False
    if cod == "E0160":
        return {"prestador_regime_mismatch_cnc": True}, False
    if group == "Plataforma":
        return {}, False
    if group == "RegraGeral":
        return {}, False
    if group in ("Prestador", "Tomador", "Valores", "IdentificacaoDps", "Intermediario", "PisCofins", "CadastroParte", "Endereco", "ServicoItem", "CatalogoMunicipal"):
        return {}, False
    return {}, True


def emit_cases(catalog: list[dict]) -> list[dict]:
    cases = []
    for c in catalog:
        mutate, pending = default_mutate(c)
        cases.append(
            {
                "cod_erro": c["cod_erro"],
                "group": c["group"],
                "validator_id": c["validator_id"],
                "expect_ok_minimal": True,
                "mutate": mutate,
                "pending": pending,
            }
        )
    return cases


def emit_dispatch(catalog: list[dict]) -> str:
    arms = []
    for c in catalog:
        fn_name = c["cod_erro"].lower()
        g = snake_group(c["group"])
        arms.append(f'        "{c["cod_erro"]}" => crate::services::nfse::rn::groups::{g}::{fn_name}(ctx),')

    return f"""//! Generated dispatch for RN rules — do not edit.
#![allow(clippy::all)]

use crate::services::nfse::dps_context::DpsContext;
use crate::services::nfse::rn::violations::RnViolation;

pub fn evaluate_cod(cod: &str, ctx: &DpsContext) -> Result<(), RnViolation> {{
    match cod {{
{chr(10).join(arms)}
        _ => Ok(()),
    }}
}}
"""


def emit_group_files(catalog: list[dict], groups_dir: Path) -> None:
    by_group: dict[str, list[dict]] = defaultdict(list)
    for c in catalog:
        by_group[c["group"]].append(c)

    mod_lines = ["//! RN rule groups (generated stubs delegate to `common`).", ""]
    for g in sorted(by_group.keys()):
        mod_lines.append(f"pub mod {snake_group(g)};")

    for g, rules in by_group.items():
        sg = snake_group(g)
        fns = []
        for r in rules:
            fn_name = r["cod_erro"].lower()
            cod = r["cod_erro"]
            fns.append(
                f'pub fn {fn_name}(ctx: &DpsContext) -> Result<(), RnViolation> {{\n'
                f'    super::common::evaluate_rule("{cod}", ctx)\n'
                f"}}"
            )
        body = f"""//! Group `{g}` — generated per-cod entrypoints.
use crate::services::nfse::dps_context::DpsContext;
use crate::services::nfse::rn::violations::RnViolation;

{chr(10).join(fns)}
"""
        (groups_dir / f"{sg}.rs").write_text(body, encoding="utf-8")

    (groups_dir / "mod.rs").write_text("\n".join(mod_lines) + "\npub mod common;\n", encoding="utf-8")
    if not (groups_dir / "common.rs").is_file():
        (groups_dir / "common.rs").write_text(
            "// Implement shared rule logic in this file (not overwritten by gen).\n",
            encoding="utf-8",
        )


def snake_group(name: str) -> str:
    out = []
    for i, ch in enumerate(name):
        if ch.isupper() and i > 0:
            out.append("_")
        out.append(ch.lower())
    return "".join(out)


def main() -> None:
    nfse = load_obrig_rej(NFSE_CSV)
    recep = load_obrig_rej(RECEP_CSV)
    catalog = merge_rules(nfse, recep)
    if len(catalog) != 425:
        print(f"WARN: expected 425 rules, got {len(catalog)}")

    OUT_JSON.parent.mkdir(parents=True, exist_ok=True)
    OUT_JSON.write_text(json.dumps(catalog, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")

    OUT_RUST.write_text(emit_rust(catalog), encoding="utf-8")

    OUT_TS.parent.mkdir(parents=True, exist_ok=True)
    OUT_TS.write_text(emit_ts(catalog), encoding="utf-8")

    OUT_DISPATCH.write_text(emit_dispatch(catalog), encoding="utf-8")
    groups_dir = ROOT / "backend/app/services/nfse/rn/groups"
    groups_dir.mkdir(parents=True, exist_ok=True)
    emit_group_files(catalog, groups_dir)

    cases = emit_cases(catalog)
    OUT_CASES.parent.mkdir(parents=True, exist_ok=True)
    OUT_CASES.write_text(json.dumps(cases, indent=2) + "\n", encoding="utf-8")
    OUT_CASES_FE.parent.mkdir(parents=True, exist_ok=True)
    OUT_CASES_FE.write_text(json.dumps(cases, indent=2) + "\n", encoding="utf-8")

    by_g = defaultdict(int)
    for c in catalog:
        by_g[c["group"]] += 1
    print(f"Wrote {len(catalog)} rules to catalog")
    print("Groups:", dict(sorted(by_g.items(), key=lambda x: -x[1])))


if __name__ == "__main__":
    main()
