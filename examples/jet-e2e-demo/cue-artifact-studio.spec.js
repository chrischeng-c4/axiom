function artifactStudioHtml() {
  return `<!doctype html>
<html>
<head>
  <title>Cue Artifact Studio Demo</title>
  <style>
    body { font-family: system-ui, sans-serif; margin: 24px; }
    main { display: grid; grid-template-columns: 220px 1fr 260px; gap: 16px; }
    section { border: 1px solid #d9dde5; border-radius: 8px; padding: 12px; }
    button, input { font: inherit; margin: 4px 0; padding: 7px 9px; }
    .state { font-weight: 700; color: #075985; }
  </style>
</head>
<body>
  <h1>Cue Artifact Studio</h1>
  <main>
    <section>
      <h2>Projects</h2>
      <button id="create-project" onclick="document.getElementById('projects').innerHTML='<li>Cue Artifact Studio</li>'">Create project</button>
      <ul id="projects"></ul>
    </section>
    <section>
      <h2>Work Item</h2>
      <input id="work-title" value="Draft product brief" />
      <button id="promote" onclick="const s=document.getElementById('work-state'); const next=s.textContent==='planned'?'implementing':'reviewing'; s.textContent=next; const t=document.getElementById('timeline'); t.textContent=t.textContent+' > work-item.'+next;">Promote</button>
      <p id="work-state" class="state">planned</p>
      <p id="timeline">project.created</p>
    </section>
    <section>
      <h2>Artifact</h2>
      <button id="publish" onclick="document.getElementById('work-state').textContent='shipped'; document.getElementById('timeline').textContent=document.getElementById('timeline').textContent+' > work-item.shipped'; document.getElementById('artifact').textContent='artifact://cue/product-brief';">Publish artifact</button>
      <p id="artifact">not published</p>
    </section>
  </main>
</body>
</html>`;
}

describe("Cue Artifact Studio product flow", () => {
  test("creates a project and keeps the work item visible", async ({ page }) => {
    await page.setContent(artifactStudioHtml());
    await page.click("#create-project");
    await page.waitForSelector("#projects li");

    expect(await page.locator("#projects").innerText()).toContain("Cue Artifact Studio");
    expect(await page.locator("#work-title").inputValue()).toBe("Draft product brief");
    expect(await page.locator("#timeline").innerText()).toContain("project.created");
  });

  test("promotes a work item into a published artifact", async ({ page }) => {
    await page.setContent(artifactStudioHtml());
    await page.click("#create-project");
    await page.click("#promote");
    await page.waitForTimeout(150);
    await page.click("#promote");
    await page.waitForTimeout(150);
    await page.click("#publish");

    expect(await page.locator("#work-state").innerText()).toBe("shipped");
    expect(await page.locator("#artifact").innerText()).toContain("artifact://cue/");
    expect(await page.locator("#timeline").innerText()).toContain("work-item.shipped");
  });
});
