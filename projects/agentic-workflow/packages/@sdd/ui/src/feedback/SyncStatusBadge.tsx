// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/interfaces/packages/@sdd/ui/src/feedback/SyncStatusBadge.md#source
// CODEGEN-BEGIN
import { cn } from '../lib/utils'
import { CheckCircle, Clock, AlertCircle, RefreshCw } from 'lucide-react'

const statusConfig: Record<string, { label: string; className: string; icon: typeof CheckCircle }> = {
  synced: { label: 'Synced', className: 'bg-green-100 text-green-800', icon: CheckCircle },
  pending: { label: 'Pending', className: 'bg-yellow-100 text-yellow-800', icon: Clock },
  failed: { label: 'Failed', className: 'bg-red-100 text-red-800', icon: AlertCircle },
}

interface SyncStatusBadgeProps {
  status: string
  className?: string
}

export default function SyncStatusBadge({ status, className }: SyncStatusBadgeProps) {
  const config = statusConfig[status] || { label: status, className: 'bg-gray-100 text-gray-700', icon: RefreshCw }
  const Icon = config.icon

  return (
    <span className={cn(
      'inline-flex items-center gap-1 rounded-full px-2.5 py-0.5 text-xs font-semibold',
      config.className,
      className,
    )}>
      <Icon className="h-3 w-3" />
      {config.label}
    </span>
  )
}


// CODEGEN-END
