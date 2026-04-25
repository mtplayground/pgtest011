import { expect, test, type Locator, type Page } from 'playwright/test';

test.describe.configure({ mode: 'serial' });

test.beforeEach(async ({ page }) => {
  await resetTodos(page);
});

test.afterEach(async ({ page }) => {
  await resetTodos(page);
});

test('adds todos and updates the counter', async ({ page }) => {
  await createTodo(page, 'write tests');
  await createTodo(page, 'ship feature');

  await expect(todoItems(page)).toHaveCount(2);
  await expect(todoLabel(page, 0)).toHaveText('write tests');
  await expect(todoLabel(page, 1)).toHaveText('ship feature');
  await expect(page.locator('.todo-count')).toContainText('2 items left');
});

test('edits todos and cancels edit with Escape', async ({ page }) => {
  await createTodo(page, 'old title');

  await todoLabel(page, 0).dblclick();
  const editInput = todoEditInput(page, 0);
  await expect(editInput).toBeFocused();
  await editInput.fill('new title');
  await editInput.press('Enter');

  await expect(todoLabel(page, 0)).toHaveText('new title');

  await todoLabel(page, 0).dblclick();
  await editInput.fill('ignored title');
  await editInput.press('Escape');

  await expect(todoLabel(page, 0)).toHaveText('new title');
});

test('deletes a todo item', async ({ page }) => {
  await createTodo(page, 'remove me');
  await createTodo(page, 'keep me');

  await todoItems(page).nth(0).hover();
  await page.locator('.todo-list li .destroy').nth(0).click({ force: true });

  await expect(todoItems(page)).toHaveCount(1);
  await expect(todoLabel(page, 0)).toHaveText('keep me');
});

test('toggles a single todo', async ({ page }) => {
  await createTodo(page, 'complete me');

  await todoToggle(page, 0).check();
  await expect(todoItems(page).nth(0)).toHaveClass(/completed/);
  await expect(page.locator('.todo-count')).toContainText('0 items left');

  await todoToggle(page, 0).uncheck();
  await expect(todoItems(page).nth(0)).not.toHaveClass(/completed/);
  await expect(page.locator('.todo-count')).toContainText('1 item left');
});

test('toggles all todos on and off', async ({ page }) => {
  await createTodo(page, 'first');
  await createTodo(page, 'second');

  const toggleAll = page.locator('#toggle-all');
  await toggleAll.check();

  await expect(todoItems(page).nth(0)).toHaveClass(/completed/);
  await expect(todoItems(page).nth(1)).toHaveClass(/completed/);
  await expect(toggleAll).toBeChecked();

  await toggleAll.uncheck();

  await expect(todoItems(page).nth(0)).not.toHaveClass(/completed/);
  await expect(todoItems(page).nth(1)).not.toHaveClass(/completed/);
  await expect(toggleAll).not.toBeChecked();
});

test('filters todos by route', async ({ page }) => {
  await createTodo(page, 'active todo');
  await createTodo(page, 'completed todo');
  await todoToggle(page, 1).check();

  await page.getByRole('link', { name: 'Active' }).click();
  await expect(page).toHaveURL(/\/active$/);
  await expect(todoItems(page)).toHaveCount(1);
  await expect(todoLabel(page, 0)).toHaveText('active todo');

  await page.getByRole('link', { name: 'Completed' }).click();
  await expect(page).toHaveURL(/\/completed$/);
  await expect(todoItems(page)).toHaveCount(1);
  await expect(todoLabel(page, 0)).toHaveText('completed todo');

  await page.getByRole('link', { name: 'All' }).click();
  await expect(page).toHaveURL(/\/$/);
  await expect(todoItems(page)).toHaveCount(2);
});

test('clears completed todos', async ({ page }) => {
  await createTodo(page, 'active todo');
  await createTodo(page, 'completed todo');
  await todoToggle(page, 1).check();

  const clearCompleted = page.getByRole('button', { name: 'Clear completed' });
  await expect(clearCompleted).toBeEnabled();
  await clearCompleted.click();

  await expect(todoItems(page)).toHaveCount(1);
  await expect(todoLabel(page, 0)).toHaveText('active todo');
  await expect(clearCompleted).toBeDisabled();
});

test('persists todos across reload', async ({ page }) => {
  await createTodo(page, 'persisted active');
  await createTodo(page, 'persisted completed');
  await todoToggle(page, 1).check();

  await page.reload();

  await expect(todoItems(page)).toHaveCount(2);
  await expect(todoLabel(page, 0)).toHaveText('persisted active');
  await expect(todoLabel(page, 1)).toHaveText('persisted completed');
  await expect(todoItems(page).nth(1)).toHaveClass(/completed/);
});

async function resetTodos(page: Page): Promise<void> {
  await page.goto('/');

  const items = todoItems(page);
  const count = await items.count();
  if (count === 0) {
    return;
  }

  const toggleAll = page.locator('#toggle-all');
  if (await toggleAll.count()) {
    await toggleAll.check();
  }

  const clearCompleted = page.getByRole('button', { name: 'Clear completed' });
  if (await clearCompleted.isVisible()) {
    await clearCompleted.click();
  }

  await expect(todoItems(page)).toHaveCount(0);
}

async function createTodo(page: Page, title: string): Promise<void> {
  await page.locator('.new-todo').fill(title);
  await page.locator('.new-todo').press('Enter');
  await expect(todoItems(page)).toContainText([title]);
}

function todoItems(page: Page): Locator {
  return page.locator('.todo-list li');
}

function todoLabel(page: Page, index: number): Locator {
  return todoItems(page).nth(index).locator('label').first();
}

function todoToggle(page: Page, index: number): Locator {
  return todoItems(page).nth(index).locator('.toggle');
}

function todoEditInput(page: Page, index: number): Locator {
  return todoItems(page).nth(index).locator('.edit');
}
