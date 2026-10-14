import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from '@/components/ui/alert-dialog'
import { Button } from '@/components/ui/button'
import { useOptionalUnsavedSave } from '@/hooks/useUnsavedFormGuard'

export function UnsavedConfirmDialog({
  open,
  onOpenChange,
  onContinueEditing,
  onDiscard,
  onSavedAndLeave,
}: {
  open: boolean
  onOpenChange: (open: boolean) => void
  onContinueEditing: () => void
  onDiscard: () => void
  onSavedAndLeave: () => void
}) {
  const saveHandler = useOptionalUnsavedSave()

  return (
    <AlertDialog open={open} onOpenChange={onOpenChange}>
      <AlertDialogContent>
        <AlertDialogHeader>
          <AlertDialogTitle>Alterações não salvas</AlertDialogTitle>
          <AlertDialogDescription>
            Existem alterações que ainda não foram salvas. Salve, continue editando ou descarte e
            saia.
          </AlertDialogDescription>
        </AlertDialogHeader>
        <AlertDialogFooter className="flex-col gap-2 sm:flex-row sm:justify-end">
          <AlertDialogCancel onClick={onContinueEditing}>Continuar editando</AlertDialogCancel>
          {saveHandler && (
            <Button
              type="button"
              onClick={async () => {
                const ok = await saveHandler()
                if (ok) onSavedAndLeave()
              }}
            >
              Salvar
            </Button>
          )}
          <AlertDialogAction onClick={onDiscard}>Descartar e sair</AlertDialogAction>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
  )
}
