// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/interfaces/packages/@sdd/ui/src/feedback/ConnectRepoForm.md#source
// CODEGEN-BEGIN
import { useState } from 'react'
import { Card, CardContent, CardHeader, CardTitle } from '../primitives/card'
import { X } from 'lucide-react'

type Platform = 'gitlab' | 'github'

export interface ConnectRepoFormData {
  platform: Platform
  // GitLab fields
  gitlab_url?: string
  gitlab_project_id?: string
  gitlab_access_token?: string
  // GitHub / generic fields
  repo_url?: string
  repo_access_token?: string
  repo_external_id?: string
  path?: string
}

/** @deprecated Use ConnectRepoFormData instead */
export interface LegacyConnectRepoFormData {
  gitlab_url: string
  gitlab_project_id: string
  gitlab_access_token: string
  path?: string
}

export interface ConnectRepoFormProps {
  onSubmit: (data: ConnectRepoFormData | LegacyConnectRepoFormData) => Promise<void>
  onClose: () => void
  isPending: boolean
  error?: string | null
}

export default function ConnectRepoForm({ onSubmit, onClose, isPending, error }: ConnectRepoFormProps) {
  const [platform, setPlatform] = useState<Platform>('gitlab')
  const [form, setForm] = useState({
    gitlab_url: '',
    gitlab_project_id: '',
    gitlab_access_token: '',
    repo_url: '',
    repo_access_token: '',
    path: '',
  })

  const isGitLabValid = platform === 'gitlab' && form.gitlab_url && form.gitlab_project_id && form.gitlab_access_token
  const isGitHubValid = platform === 'github' && form.repo_url && form.repo_access_token
  const isValid = isGitLabValid || isGitHubValid

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()

    if (platform === 'gitlab') {
      await onSubmit({
        platform: 'gitlab',
        gitlab_url: form.gitlab_url,
        gitlab_project_id: form.gitlab_project_id,
        gitlab_access_token: form.gitlab_access_token,
        repo_url: form.gitlab_url,
        repo_external_id: form.gitlab_project_id,
        repo_access_token: form.gitlab_access_token,
        path: form.path || undefined,
      })
    } else {
      await onSubmit({
        platform: 'github',
        repo_url: form.repo_url,
        repo_access_token: form.repo_access_token,
        path: form.path || undefined,
      })
    }
  }

  const platformLabel = platform === 'gitlab' ? 'GitLab' : 'GitHub'

  return (
    <Card>
      <CardHeader>
        <div className="flex items-center justify-between">
          <CardTitle>Connect Repository</CardTitle>
          <button
            type="button"
            onClick={onClose}
            className="text-gray-400 hover:text-gray-600 cursor-pointer"
          >
            <X className="h-5 w-5" />
          </button>
        </div>
        <p className="text-sm text-gray-500 mt-1">
          Connect a {platformLabel} repository to this project. Issues will be imported automatically.
        </p>
      </CardHeader>
      <CardContent>
        <form onSubmit={handleSubmit} className="space-y-4">
          {/* Platform selector */}
          <div>
            <label htmlFor="platform" className="block text-sm font-medium text-gray-700 mb-1">
              Platform *
            </label>
            <select
              id="platform"
              value={platform}
              onChange={(e) => setPlatform(e.target.value as Platform)}
              className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-primary focus:border-transparent text-sm bg-white"
            >
              <option value="gitlab">GitLab</option>
              <option value="github">GitHub</option>
            </select>
          </div>

          {/* GitLab-specific fields */}
          {platform === 'gitlab' && (
            <>
              <div>
                <label htmlFor="gitlab_url" className="block text-sm font-medium text-gray-700 mb-1">
                  GitLab URL *
                </label>
                <input
                  id="gitlab_url"
                  type="url"
                  required
                  value={form.gitlab_url}
                  onChange={(e) => setForm({ ...form, gitlab_url: e.target.value })}
                  className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-primary focus:border-transparent text-sm"
                  placeholder="https://gitlab.com"
                />
              </div>
              <div>
                <label htmlFor="gitlab_project_id" className="block text-sm font-medium text-gray-700 mb-1">
                  GitLab Project ID *
                </label>
                <input
                  id="gitlab_project_id"
                  type="text"
                  required
                  value={form.gitlab_project_id}
                  onChange={(e) => setForm({ ...form, gitlab_project_id: e.target.value })}
                  className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-primary focus:border-transparent text-sm"
                  placeholder="12345"
                />
                <p className="text-xs text-gray-400 mt-1">Found in Settings &rarr; General on your GitLab project page.</p>
              </div>
              <div>
                <label htmlFor="gitlab_access_token" className="block text-sm font-medium text-gray-700 mb-1">
                  Access Token *
                </label>
                <input
                  id="gitlab_access_token"
                  type="password"
                  required
                  value={form.gitlab_access_token}
                  onChange={(e) => setForm({ ...form, gitlab_access_token: e.target.value })}
                  className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-primary focus:border-transparent text-sm"
                  placeholder="glpat-..."
                />
                <p className="text-xs text-gray-400 mt-1">Project or personal access token with api scope.</p>
              </div>
            </>
          )}

          {/* GitHub-specific fields */}
          {platform === 'github' && (
            <>
              <div>
                <label htmlFor="repo_url" className="block text-sm font-medium text-gray-700 mb-1">
                  Repository URL *
                </label>
                <input
                  id="repo_url"
                  type="url"
                  required
                  value={form.repo_url}
                  onChange={(e) => setForm({ ...form, repo_url: e.target.value })}
                  className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-primary focus:border-transparent text-sm"
                  placeholder="https://github.com/owner/repo"
                />
              </div>
              <div>
                <label htmlFor="repo_access_token" className="block text-sm font-medium text-gray-700 mb-1">
                  Access Token *
                </label>
                <input
                  id="repo_access_token"
                  type="password"
                  required
                  value={form.repo_access_token}
                  onChange={(e) => setForm({ ...form, repo_access_token: e.target.value })}
                  className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-primary focus:border-transparent text-sm"
                  placeholder="ghp_..."
                />
                <p className="text-xs text-gray-400 mt-1">GitHub personal access token with repo scope.</p>
              </div>
            </>
          )}

          {/* Shared path field */}
          <div>
            <label htmlFor="repo_path" className="block text-sm font-medium text-gray-700 mb-1">
              Path
            </label>
            <input
              id="repo_path"
              type="text"
              value={form.path}
              onChange={(e) => setForm({ ...form, path: e.target.value })}
              className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-primary focus:border-transparent text-sm"
              placeholder="/ (root) or subdirectory path"
            />
          </div>

          {error && (
            <div role="alert" className="p-3 bg-red-50 border border-red-200 rounded-lg text-sm text-red-700">
              {error}
            </div>
          )}

          <div className="flex justify-end gap-3 pt-2">
            <button
              type="button"
              onClick={onClose}
              className="px-4 py-2 border border-gray-300 rounded-lg text-sm font-medium text-gray-700 hover:bg-gray-50 transition-colors cursor-pointer"
            >
              Cancel
            </button>
            <button
              type="submit"
              disabled={isPending || !isValid}
              className="px-4 py-2 bg-primary text-primary-foreground rounded-lg text-sm font-medium hover:bg-primary/90 transition-colors disabled:opacity-50 disabled:cursor-not-allowed cursor-pointer"
            >
              {isPending ? 'Connecting...' : 'Connect Repo'}
            </button>
          </div>
        </form>
      </CardContent>
    </Card>
  )
}


// CODEGEN-END
