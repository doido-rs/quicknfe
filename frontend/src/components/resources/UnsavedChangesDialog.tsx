import { UnsavedConfirmDialog } from '@/components/resources/UnsavedConfirmDialog'
import { useUnsavedNavigationBlocker } from '@/hooks/useUnsavedFormGuard'

export function UnsavedChangesDialog() {
  const blocker = useUnsavedNavigationBlocker()

  if (blocker.status !== 'blocked') {
    return null
  }

  return (
    <UnsavedConfirmDialog
      open
      onOpenChange={(open) => !open && blocker.reset()}
      onContinueEditing={blocker.reset}
      onDiscard={blocker.proceed}
      onSavedAndLeave={blocker.proceed}
    />
  )
}
