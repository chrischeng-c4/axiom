// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/interfaces/packages/@sdd/ui/src/changes/CommentsSection.md#source
// CODEGEN-BEGIN
import { Card, CardContent, CardHeader, CardTitle } from '../primitives/card'
import { MessageSquare } from 'lucide-react'
import { formatDistanceToNow } from 'date-fns'
import ReactMarkdown from 'react-markdown'
import remarkGfm from 'remark-gfm'
import type { IssueComment } from '../types'

export interface CommentsSectionProps {
  comments: IssueComment[]
  isLoading: boolean
}

export default function CommentsSection({ comments, isLoading }: CommentsSectionProps) {
  if (isLoading) {
    return (
      <Card>
        <CardContent className="pt-6">
          <div className="h-8 bg-gray-200 rounded animate-pulse" />
        </CardContent>
      </Card>
    )
  }

  if (!comments || comments.length === 0) return null

  return (
    <Card>
      <CardHeader className="pb-3">
        <CardTitle className="text-sm flex items-center gap-2">
          <MessageSquare className="h-4 w-4" />
          Comments ({comments.length})
        </CardTitle>
      </CardHeader>
      <CardContent className="space-y-4">
        {comments.map((comment) => (
          <div key={comment.id} className="border-b last:border-b-0 pb-4 last:pb-0">
            <div className="flex items-center gap-2 mb-2">
              <span className="text-sm font-medium text-gray-900">@{comment.author.username}</span>
              <span className="text-xs text-gray-400">
                {formatDistanceToNow(new Date(comment.created_at), { addSuffix: true })}
              </span>
            </div>
            <div className="prose prose-sm max-w-none prose-p:text-gray-700 prose-a:text-primary prose-code:text-sm prose-code:before:content-none prose-code:after:content-none prose-pre:bg-gray-50 prose-pre:border prose-pre:border-gray-200">
              <ReactMarkdown remarkPlugins={[remarkGfm]}>
                {comment.body}
              </ReactMarkdown>
            </div>
          </div>
        ))}
      </CardContent>
    </Card>
  )
}


// CODEGEN-END
