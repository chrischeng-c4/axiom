// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/interfaces/packages/@sdd/ui/src/file-browser/FileViewer.md#source
// CODEGEN-BEGIN
import { useMemo } from 'react'
import { Prism as SyntaxHighlighter } from 'react-syntax-highlighter'
import { oneDark } from 'react-syntax-highlighter/dist/esm/styles/prism'
import Papa from 'papaparse'
import * as XLSX from 'xlsx'
import { FileText, Download } from 'lucide-react'
import type { FileContent } from '../types'

interface FileViewerProps {
  file: FileContent
}

const EXT_TO_LANGUAGE: Record<string, string> = {
  ts: 'typescript', tsx: 'tsx', js: 'javascript', jsx: 'jsx',
  py: 'python', rb: 'ruby', go: 'go', rs: 'rust',
  java: 'java', kt: 'kotlin', scala: 'scala',
  c: 'c', h: 'c', cpp: 'cpp', hpp: 'cpp', cs: 'csharp',
  swift: 'swift', m: 'objectivec',
  sh: 'bash', bash: 'bash', zsh: 'bash',
  sql: 'sql', graphql: 'graphql',
  html: 'html', htm: 'html', css: 'css', scss: 'scss', sass: 'sass', less: 'less',
  json: 'json', yaml: 'yaml', yml: 'yaml', toml: 'toml', xml: 'xml',
  md: 'markdown', rst: 'markdown',
  dockerfile: 'dockerfile', makefile: 'makefile',
  proto: 'protobuf', tf: 'hcl',
  lua: 'lua', r: 'r', pl: 'perl', ex: 'elixir', exs: 'elixir',
  erl: 'erlang', hs: 'haskell', clj: 'clojure',
  vue: 'html', svelte: 'html', php: 'php', dart: 'dart',
  env: 'bash', gitignore: 'bash',
}

const IMAGE_EXTENSIONS = new Set(['png', 'jpg', 'jpeg', 'gif', 'svg', 'webp', 'ico'])
const VIDEO_EXTENSIONS = new Set(['mp4', 'webm', 'mov'])

function getExtension(filename: string): string {
  const parts = filename.split('.')
  if (parts.length < 2) return filename.toLowerCase()
  return parts[parts.length - 1].toLowerCase()
}

export default function FileViewer({ file }: FileViewerProps) {
  const ext = getExtension(file.file_name)

  // Image
  if (IMAGE_EXTENSIONS.has(ext)) {
    const src = file.encoding === 'base64'
      ? `data:${file.mime_type || 'image/png'};base64,${file.content}`
      : `data:image/svg+xml;utf8,${encodeURIComponent(file.content)}`
    return (
      <div className="flex items-center justify-center p-8 bg-gray-50 rounded-lg">
        <img src={src} alt={file.file_name} className="max-w-full max-h-[600px] object-contain" />
      </div>
    )
  }

  // Video
  if (VIDEO_EXTENSIONS.has(ext)) {
    const src = `data:${file.mime_type || 'video/mp4'};base64,${file.content}`
    return (
      <div className="flex items-center justify-center p-4 bg-gray-50 rounded-lg">
        <video controls className="max-w-full max-h-[600px]">
          <source src={src} type={file.mime_type || 'video/mp4'} />
        </video>
      </div>
    )
  }

  // PDF
  if (ext === 'pdf') {
    const src = `data:application/pdf;base64,${file.content}`
    return (
      <div className="rounded-lg overflow-hidden border">
        <iframe src={src} title={file.file_name} className="w-full h-[700px]" />
      </div>
    )
  }

  // CSV / TSV
  if (ext === 'csv' || ext === 'tsv') {
    return <CsvViewer content={file.content} delimiter={ext === 'tsv' ? '\t' : ','} />
  }

  // Excel
  if (ext === 'xlsx' || ext === 'xls') {
    return <ExcelViewer content={file.content} />
  }

  // Code / text
  const language = EXT_TO_LANGUAGE[ext]
  if (file.encoding === 'text') {
    return (
      <div className="rounded-lg overflow-hidden border text-sm">
        <SyntaxHighlighter
          language={language || 'text'}
          style={oneDark}
          showLineNumbers
          customStyle={{ margin: 0, borderRadius: 0 }}
        >
          {file.content}
        </SyntaxHighlighter>
      </div>
    )
  }

  // Fallback -- unknown binary
  return <FallbackViewer file={file} />
}

function CsvViewer({ content, delimiter }: { content: string; delimiter: string }) {
  const parsed = useMemo(() => {
    const result = Papa.parse(content, { delimiter, header: false })
    return result.data as string[][]
  }, [content, delimiter])

  if (parsed.length === 0) return <p className="text-sm text-gray-500">Empty file</p>

  const headers = parsed[0]
  const rows = parsed.slice(1).filter(row => row.some(cell => cell !== ''))

  return (
    <div className="border rounded-lg overflow-auto max-h-[600px]">
      <table className="w-full text-sm">
        <thead className="bg-gray-50 sticky top-0">
          <tr>
            {headers.map((h, i) => (
              <th key={i} className="px-3 py-2 text-left font-medium text-gray-700 border-b whitespace-nowrap">
                {h}
              </th>
            ))}
          </tr>
        </thead>
        <tbody>
          {rows.map((row, ri) => (
            <tr key={ri} className="hover:bg-gray-50">
              {row.map((cell, ci) => (
                <td key={ci} className="px-3 py-1.5 border-b text-gray-600 whitespace-nowrap">
                  {cell}
                </td>
              ))}
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  )
}

function ExcelViewer({ content }: { content: string }) {
  const rows = useMemo(() => {
    try {
      const workbook = XLSX.read(content, { type: 'base64' })
      const firstSheet = workbook.Sheets[workbook.SheetNames[0]]
      const data = XLSX.utils.sheet_to_json<string[]>(firstSheet, { header: 1 })
      return data
    } catch {
      return null
    }
  }, [content])

  if (!rows) return <p className="text-sm text-gray-500">Unable to parse Excel file</p>
  if (rows.length === 0) return <p className="text-sm text-gray-500">Empty spreadsheet</p>

  const headers = rows[0]
  const bodyRows = rows.slice(1)

  return (
    <div className="border rounded-lg overflow-auto max-h-[600px]">
      <table className="w-full text-sm">
        <thead className="bg-gray-50 sticky top-0">
          <tr>
            {headers.map((h, i) => (
              <th key={i} className="px-3 py-2 text-left font-medium text-gray-700 border-b whitespace-nowrap">
                {String(h)}
              </th>
            ))}
          </tr>
        </thead>
        <tbody>
          {bodyRows.map((row, ri) => (
            <tr key={ri} className="hover:bg-gray-50">
              {(row as unknown[]).map((cell, ci) => (
                <td key={ci} className="px-3 py-1.5 border-b text-gray-600 whitespace-nowrap">
                  {String(cell ?? '')}
                </td>
              ))}
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  )
}

function FallbackViewer({ file }: { file: FileContent }) {
  const downloadUrl = file.encoding === 'base64'
    ? `data:${file.mime_type || 'application/octet-stream'};base64,${file.content}`
    : `data:text/plain;charset=utf-8,${encodeURIComponent(file.content)}`

  return (
    <div className="flex flex-col items-center justify-center py-16 text-gray-500">
      <FileText className="h-12 w-12 mb-4 text-gray-300" />
      <p className="text-sm font-medium">{file.file_name}</p>
      <p className="text-xs mt-1">{formatSize(file.size)}</p>
      <a
        href={downloadUrl}
        download={file.file_name}
        className="mt-4 inline-flex items-center gap-2 px-4 py-2 bg-gray-100 rounded-lg text-sm hover:bg-gray-200 transition-colors"
      >
        <Download className="h-4 w-4" />
        Download
      </a>
    </div>
  )
}

function formatSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`
}


// CODEGEN-END
