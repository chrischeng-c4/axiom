// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/interfaces/packages/@sdd/ui/src/changes/ChangeList.md#source
// CODEGEN-BEGIN
import { ExternalLink, GitBranch, FileText } from 'lucide-react';
import type { Change } from '../types';

const STATUS_COLORS: Record<string, string> = {
  draft: 'bg-gray-100 text-gray-700',
  in_progress: 'bg-blue-100 text-blue-700',
  review: 'bg-yellow-100 text-yellow-700',
  merged: 'bg-green-100 text-green-700',
  closed: 'bg-red-100 text-red-700',
};

interface ChangeListProps {
  changes: Change[];
  onSelect: (change: Change) => void;
  selectedId?: string;
}

export function ChangeList({ changes, onSelect, selectedId }: ChangeListProps) {
  if (changes.length === 0) {
    return (
      <div className="text-center py-12 text-gray-500">
        <FileText className="mx-auto h-12 w-12 mb-4 opacity-50" />
        <p>No changes yet</p>
        <p className="text-sm mt-1">Select issues and create a change to get started</p>
      </div>
    );
  }

  return (
    <div className="divide-y divide-gray-200">
      {changes.map((change) => (
        <div
          key={change.id}
          onClick={() => onSelect(change)}
          className={`p-4 cursor-pointer hover:bg-gray-50 transition-colors ${
            selectedId === change.id ? 'bg-blue-50 border-l-2 border-blue-500' : ''
          }`}
        >
          <div className="flex items-start justify-between">
            <div className="flex-1 min-w-0">
              <h3 className="text-sm font-medium text-gray-900 truncate">{change.title}</h3>
              <div className="flex items-center gap-2 mt-1">
                <span className={`inline-flex items-center px-2 py-0.5 rounded text-xs font-medium ${STATUS_COLORS[change.status] || STATUS_COLORS.draft}`}>
                  {change.status.replace('_', ' ')}
                </span>
                <span className="text-xs text-gray-500">
                  {change.issue_ids.length} issue{change.issue_ids.length !== 1 ? 's' : ''}
                </span>
                {change.spec_ids.length > 0 && (
                  <span className="text-xs text-gray-500">
                    {change.spec_ids.length} spec{change.spec_ids.length !== 1 ? 's' : ''}
                  </span>
                )}
              </div>
            </div>
            <div className="flex items-center gap-2 ml-2">
              {change.branch_name && (
                <span className="inline-flex items-center text-xs text-gray-500">
                  <GitBranch className="h-3 w-3 mr-1" />
                  {change.branch_name}
                </span>
              )}
              {change.external_mr_url && (
                <a
                  href={change.external_mr_url}
                  target="_blank"
                  rel="noopener noreferrer"
                  onClick={(e) => e.stopPropagation()}
                  className="text-blue-500 hover:text-blue-700"
                >
                  <ExternalLink className="h-4 w-4" />
                </a>
              )}
            </div>
          </div>
          <p className="text-xs text-gray-400 mt-1">
            {new Date(change.created_at).toLocaleDateString()}
          </p>
        </div>
      ))}
    </div>
  );
}


// CODEGEN-END
