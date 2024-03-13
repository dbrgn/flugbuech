import {expect, test} from '@playwright/test';

test('index page welcomes guest when not logged in', async ({page}) => {
    await page.goto('/');
    await expect(page.getByText('Welcome, Guest')).toBeVisible();
});

test('index page welcomes user when logged in', async ({page, context}) => {
    context.addCookies([{name: 'user_name', value: 'Chrigel', domain: 'localhost', path: '/'}]);
    await page.goto('/');
    await expect(page.getByText('Welcome, Chrigel')).toBeVisible();
});
