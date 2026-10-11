import { Link } from '@tanstack/react-router'
import { useAuth } from '@/auth/AuthContext'
import { regimeTributarioLabel } from '@/lib/regimeTributario'
import { resourceShowTo } from '@/lib/resourcePaths'
import { cn } from '@/lib/utils'
import { resources } from '@/resources/definitions'

export function PrestadorSessionPanel({
  title,
  className,
}: {
  title: string
  className?: string
}) {
  const { me } = useAuth()
  const prestador = me?.prestador_ativo

  return (
    <section
      className={cn('rounded-lg border bg-card p-4 md:p-6', className)}
      aria-labelledby="prestador-form-heading"
    >
      <h2 id="prestador-form-heading" className="text-lg font-semibold tracking-tight">
        {title}
      </h2>
      {!prestador ? (
        <p className="mt-3 text-sm text-muted-foreground">
          Nenhum prestador selecionado na sessão.{' '}
          <Link to="/tenant/prestador" className="text-primary underline-offset-4 hover:underline">
            Selecionar prestador
          </Link>
        </p>
      ) : (
        <dl className="mt-4 grid gap-3 text-sm sm:grid-cols-2 lg:grid-cols-3">
          <div>
            <dt className="font-medium text-muted-foreground">Razão social</dt>
            <dd>{prestador.razao_social}</dd>
          </div>
          <div>
            <dt className="font-medium text-muted-foreground">CPF/CNPJ</dt>
            <dd>{prestador.cpf_cnpj}</dd>
          </div>
          {'regime_tributario' in prestador && prestador.regime_tributario != null && (
            <div>
              <dt className="font-medium text-muted-foreground">Regime tributário</dt>
              <dd>{regimeTributarioLabel(prestador.regime_tributario)}</dd>
            </div>
          )}
          {prestador.municipio && (
            <div>
              <dt className="font-medium text-muted-foreground">Município</dt>
              <dd>
                {(() => {
                  const municipiosDef = resources.find((r) => r.key === 'municipios')
                  if (municipiosDef && prestador.municipio?.id != null) {
                    return (
                      <Link
                        {...resourceShowTo(municipiosDef, prestador.municipio.id)}
                        className="text-primary underline-offset-4 hover:underline"
                      >
                        {prestador.municipio.nome}
                      </Link>
                    )
                  }
                  return prestador.municipio.nome
                })()}
                <span className="mt-0.5 block text-xs text-muted-foreground">
                  UF, IBGE e demais dados no cadastro de municípios.
                </span>
              </dd>
            </div>
          )}
        </dl>
      )}
    </section>
  )
}
