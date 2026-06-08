---
id: projects-sdd-packages-sdd-ui-src-changes-commentssection-tsx
fill_sections: [overview, source, changes]
capability_refs:
  - id: aw-core-client-model-workitem-first-artifact-lifecycle
    role: primary
    gap: client-boundary-model
    claim: client-boundary-model
    coverage: full
    rationale: "The @sdd package interfaces are client embedding surfaces over the AW Core workflow protocol."
---

# Standardized projects/agentic-workflow/packages/@sdd/ui/src/changes/CommentsSection.tsx

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/packages/@sdd/ui/src/changes/CommentsSection.tsx` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `CommentsSectionProps` | projects/agentic-workflow/packages/@sdd/ui/src/changes/CommentsSection.tsx | interface | pub | 10 |  |
## Source
<!-- type: source lang: tsx -->

```tsx
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

```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/packages/@sdd/ui/src/changes/CommentsSection.tsx
    action: create
    section: source
    impl_mode: codegen
    description: |
      Generate the complete source file from the generic Source template. This
      is a cross-language raw source template used for regenerable adoption
      before higher-level semantic generators exist.
```
