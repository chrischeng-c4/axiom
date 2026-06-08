// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/interfaces/packages/@sdd/ui/src/feedback/ConfirmDialog.md#source
// CODEGEN-BEGIN
/**
 * ConfirmDialog -- lightweight modal for destructive-action confirmation.
 * Uses a React portal so it renders above all other content.
 */
import { useEffect, useRef } from 'react'
import { createPortal } from 'react-dom'
import { AlertTriangle } from 'lucide-react'

interface ConfirmDialogProps {
  open: boolean
  title: string
  description: string
  confirmLabel?: string
  cancelLabel?: string
  variant?: 'danger' | 'warning'
  onConfirm: () => void
  onCancel: () => void
}

export default function ConfirmDialog({
  open,
  title,
  description,
  confirmLabel = 'Confirm',
  cancelLabel = 'Cancel',
  variant = 'danger',
  onConfirm,
  onCancel,
}: ConfirmDialogProps) {
  const confirmBtnRef = useRef<HTMLButtonElement>(null)

  // Focus confirm button when opened; trap Escape to cancel
  useEffect(() => {
    if (!open) return
    confirmBtnRef.current?.focus()
    const onKey = (e: KeyboardEvent) => { if (e.key === 'Escape') onCancel() }
    window.addEventListener('keydown', onKey)
    return () => window.removeEventListener('keydown', onKey)
  }, [open, onCancel])

  if (!open) return null

  const confirmCls =
    variant === 'danger'
      ? 'bg-red-600 hover:bg-red-700 focus-visible:ring-red-500 text-white'
      : 'bg-amber-500 hover:bg-amber-600 focus-visible:ring-amber-400 text-white'

  const iconCls =
    variant === 'danger' ? 'text-red-500' : 'text-amber-500'

  return createPortal(
    /* Backdrop */
    <div
      className="fixed inset-0 z-50 flex items-center justify-center p-4"
      aria-modal="true"
      role="dialog"
      aria-labelledby="confirm-dialog-title"
    >
      {/* Scrim */}
      <div
        className="absolute inset-0 bg-black/40 backdrop-blur-[2px] animate-in fade-in duration-150"
        onClick={onCancel}
      />

      {/* Panel */}
      <div className="relative z-10 w-full max-w-sm rounded-xl bg-white shadow-2xl ring-1 ring-black/5 animate-in zoom-in-95 fade-in duration-150">
        {/* Body */}
        <div className="flex gap-4 p-6">
          <div className={`mt-0.5 flex-shrink-0 ${iconCls}`}>
            <AlertTriangle className="h-5 w-5" />
          </div>
          <div className="min-w-0">
            <h2
              id="confirm-dialog-title"
              className="text-sm font-semibold text-gray-900 leading-snug"
            >
              {title}
            </h2>
            <p className="mt-1.5 text-sm text-gray-500 leading-relaxed">
              {description}
            </p>
          </div>
        </div>

        {/* Footer */}
        <div className="flex justify-end gap-2 border-t border-gray-100 px-6 py-4">
          <button
            onClick={onCancel}
            className="rounded-lg px-4 py-2 text-sm font-medium text-gray-700 bg-gray-100 hover:bg-gray-200 transition-colors duration-150 cursor-pointer focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-gray-400"
          >
            {cancelLabel}
          </button>
          <button
            ref={confirmBtnRef}
            onClick={onConfirm}
            className={`rounded-lg px-4 py-2 text-sm font-medium transition-colors duration-150 cursor-pointer focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-offset-2 ${confirmCls}`}
          >
            {confirmLabel}
          </button>
        </div>
      </div>
    </div>,
    document.body,
  )
}


// CODEGEN-END
