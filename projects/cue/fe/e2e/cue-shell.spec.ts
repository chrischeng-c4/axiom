import { expect, test } from '@jet/test'

const baseUrl = 'http://127.0.0.1:3212'

function collectUnexpectedConsoleErrors(page): string[] {
  const errors: string[] = []
  page.on('console', (msg) => {
    if (msg.type() !== 'error') return
    const text = msg.text()
    if (text.includes('Failed to load resource')) return
    errors.push(text)
  })
  return errors
}

test.describe('Cue control plane', () => {
  test('loads a Sandbox deep link with owner-safe complexity', async ({ page }) => {
    const errors = collectUnexpectedConsoleErrors(page)

    await page.goto(`${baseUrl}/apps/t-operations-team-request-tracker/sandbox`)

    expect(await page.locator('h1').textContent()).toBe('Governed Approval Tracker')
    const body = await page.locator('body').innerText()
    expect(body).toContain('你只要說明需求，Cue 會整理成可審核的 App')
    expect(body).toContain('試用重點')
    expect(body).toContain('用範例資料試跑流程')
    expect(body.includes('Hidden GitLab project')).toBe(false)
    expect(body.includes('Runtime tenant')).toBe(false)
    expect(body).toContain('Sandbox 是 owner 驗收區')
    expect(errors).toHaveLength(0)
  })

  test('moves from Studio preview to Sandbox review', async ({ page }) => {
    const errors = collectUnexpectedConsoleErrors(page)

    await page.goto(`${baseUrl}/apps/t-operations-team-request-tracker/studio`)

    let body = await page.locator('body').innerText()
    expect(body).toContain('確認 App 預覽')
    expect(body).toContain('申請 Sandbox')

    await page.evaluate(() => {
      const sandboxButton = Array.from(document.querySelectorAll('button')).find((button) =>
        button.textContent?.includes('申請 Sandbox'),
      )
      sandboxButton?.click()
    })

    await page.waitForTimeout(100)
    expect(await page.url()).toMatch(/\/apps\/t-operations-team-request-tracker\/sandbox$/)
    body = await page.locator('body').innerText()
    expect(body).toContain('試用重點')
    expect(body).toContain('Sandbox 是 owner 驗收區')
    expect(body).toContain('Sandbox ready')
    expect(errors).toHaveLength(0)
  })

  test('shows Admin workspace review tickets for deployment SaaS API and costly resources', async ({ page }) => {
    const errors = collectUnexpectedConsoleErrors(page)

    await page.goto(`${baseUrl}/admin`)

    let body = await page.locator('body').innerText()
    expect(body).toContain('Admin workspace')
    expect(body).toContain('Platform artifacts')
    expect(body).toContain('Hidden GitLab project')
    expect(body).toContain('Runtime tenant binding')
    expect(body).toContain('待審 Review tickets')
    expect(body).toContain('Deploy test build and publish Sandbox result')
    expect(body).toContain('Enable Slack notification API')
    expect(body).toContain('Training model for request triage')
    expect(body).toContain('3 張 Admin ticket 待審')

    await page.evaluate(() => {
      const approveButton = Array.from(document.querySelectorAll('button')).find((button) =>
        button.textContent?.includes('核准 ticket'),
      )
      approveButton?.click()
    })

    await page.waitForTimeout(100)
    body = await page.locator('body').innerText()
    expect(body).toContain('Ticket 已核准')
    expect(body).toContain('2 張 Admin ticket 待審')
    expect(errors).toHaveLength(0)
  })
})
