import { Link, useNavigate, type LinkProps } from '@tanstack/react-router'
import { UnsavedConfirmDialog } from '@/components/resources/UnsavedConfirmDialog'
import { useGuardedNavigate } from '@/hooks/useUnsavedFormGuard'

type GuardedLinkProps = LinkProps & {
  children: React.ReactNode
  className?: string
  onClick?: React.MouseEventHandler<HTMLAnchorElement>
}

export function GuardedLink({ children, onClick, ...props }: GuardedLinkProps) {
  const navigate = useNavigate()
  const {
    requestNavigation,
    pendingConfirmation,
    confirmDiscard,
    cancelNavigation,
    confirmSavedAndLeave,
  } = useGuardedNavigate()

  return (
    <>
      <Link
        {...props}
        onClick={(e) => {
          e.preventDefault()
          requestNavigation(() => {
            void navigate({
              to: props.to,
              params: props.params,
              search: props.search,
            })
          })
          onClick?.(e)
        }}
      >
        {children}
      </Link>
      <UnsavedConfirmDialog
        open={pendingConfirmation}
        onOpenChange={(open) => !open && cancelNavigation()}
        onContinueEditing={cancelNavigation}
        onDiscard={confirmDiscard}
        onSavedAndLeave={confirmSavedAndLeave}
      />
    </>
  )
}
