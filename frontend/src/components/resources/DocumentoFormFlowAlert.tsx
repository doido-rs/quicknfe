import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert'

export function DocumentoCreateFlowAlert() {
  return (
    <Alert>
      <AlertTitle>Próximos passos</AlertTitle>
      <AlertDescription className="space-y-1">
        <p>
          Informe um valor da operação maior que zero e salve o cabeçalho. Em seguida, use{' '}
          <strong>Editar</strong> para incluir os <strong>itens de serviço</strong>.
        </p>
        <p className="text-muted-foreground">
          Para assinar, a nota precisa de tomador (quando Brasil), itens, tributação coerente e demais
          regras da DPS.
        </p>
      </AlertDescription>
    </Alert>
  )
}

export function DocumentoEditFlowAlert({ itensCount }: { itensCount?: number }) {
  const itensLabel =
    itensCount == null ? null : itensCount === 0 ? 'Nenhum item cadastrado ainda.' : `${itensCount} item(ns) cadastrado(s).`

  return (
    <Alert>
      <AlertTitle>Itens e assinatura</AlertTitle>
      <AlertDescription className="space-y-1">
        {itensLabel && <p>{itensLabel}</p>}
        <p>
          Cadastre os itens na aba <strong>Itens</strong> (entre Partes e Valores). A aba{' '}
          <strong>Valores</strong> usa a soma dos totais brutos dos itens como valor da operação; salve
          o cabeçalho para persistir.
        </p>
        <p className="text-muted-foreground">
          Revise tomador, competência e tributos antes de assinar na tela de visualização.
        </p>
      </AlertDescription>
    </Alert>
  )
}
