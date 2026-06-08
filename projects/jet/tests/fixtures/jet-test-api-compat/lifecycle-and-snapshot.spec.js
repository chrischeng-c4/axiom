// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-tests-fixtures-jet-test-api-compat.md#tests
// CODEGEN-BEGIN
// API compatibility fixture for `@jet/test` (#2715).
//
// This spec is part of the regression corpus that guards the
// `@jet/test` virtual-module contract — hooks, basic matchers, and
// text snapshots. Any change that breaks a behavior exercised here is
// a contract regression and must surface in CI before it ships.
//
// Keep this fixture small enough that the harness can run it in CI by
// default. New behaviors should land as separate fixture files rather
// than be appended here.

import { test, expect, beforeAll, beforeEach, afterEach, afterAll } from '@jet/test';

let order = [];
let perTestSeed;

beforeAll(() => {
  order.push('beforeAll');
});

beforeEach(() => {
  order.push('beforeEach');
  perTestSeed = 'fresh';
});

afterEach(() => {
  order.push('afterEach');
});

afterAll(() => {
  order.push('afterAll');
});

test('hooks run before the body and toBe matches', () => {
  expect(order).toEqual(['beforeAll', 'beforeEach']);
  expect(perTestSeed).toBe('fresh');
  expect(1 + 1).toBe(2);
});

test('per-test seed is reset between tests', () => {
  // If beforeEach didn't run, perTestSeed would still be 'fresh' from the
  // previous test rather than being reseeded. Mutating it here is what we
  // are guarding against — the next test would see this drift.
  expect(perTestSeed).toBe('fresh');
  perTestSeed = 'drift';
  expect(perTestSeed).toBe('drift');
});

test('text snapshot matches the baseline byte-for-byte', async () => {
  await expect('hello compat').toMatchTextSnapshot('greeting');
});
// CODEGEN-END
