---
name: vue-expert
description: Master Vue 3 engineering skill supporting both TypeScript and Vanilla JavaScript with JSDoc typing. Builds Vue 3 components with Composition API patterns, configures Nuxt 3 SSR/SSG projects, sets up Pinia stores, scaffolds Quasar/Capacitor mobile apps, implements PWA features, optimizes Vite builds, and provides full type coverage without TS compiler using JSDoc. Use when creating or optimizing Vue 3 applications, writing reusable composables, managing state with Pinia, building hybrid mobile apps, configuring service workers, tuning Vite builds, or when migrating/working with Vanilla JS and JSDoc in Vue 3.
license: MIT
metadata:
  author: https://github.com/Jeffallan
  company: https://synergetic.solutions
  version: "2.0.0"
  domain: frontend
  triggers: Vue, Vue 3, Composition API, Nuxt, Pinia, Vue composables, reactive, ref, Vue Router, Vite Vue, Quasar, Capacitor, PWA, service worker, Fastify SSR, sourcemap, Vite config, build optimization, Vue JavaScript, Vue without TypeScript, Vue JSDoc, Vue JS only, Vue vanilla JavaScript, .mjs Vue, Vue no TS
  role: master-specialist
  scope: implementation-and-architecture
  output-format: code
  related-skills: typescript-pro, javascript-pro, fullstack-guardian
---

# Vue Expert (Unified TypeScript & JavaScript Edition)

Comprehensive Vue 3 specialist with deep expertise in the Composition API, reactivity system, TypeScript integration, and vanilla JavaScript with JSDoc typing.

---

## Mode Detection: TypeScript vs. Vanilla JavaScript

Before scaffolding or generating code, check the project setup:

1. **TypeScript Mode** (Default if `tsconfig.json` or `.ts` files exist):
   - Use `<script setup lang="ts">`
   - Use typed props via `defineProps<{ ... }>()` and generic emits `defineEmits<{ ... }>()`
   - Validate with `vue-tsc --noEmit`

2. **Vanilla JavaScript / JSDoc Mode** (If `jsconfig.json`, `.mjs`, or no `tsconfig.json` exists, or explicitly requested):
   - Use `<script setup>` (no `lang="ts"`)
   - Use runtime props with `defineProps({ ... })` enriched with comprehensive JSDoc `@typedef`, `@type`, and `@property`
   - Annotate composables with `@param` and `@returns` for full IDE autocomplete and type safety
   - Validate using `eslint-plugin-jsdoc`

---

## Core Workflow

1. **Analyze Requirements & Environment** — Identify component hierarchy, state needs, routing, and verify whether TS or Vanilla JS/JSDoc is the project standard.
2. **Design Architecture** — Plan composables, Pinia stores, component structure, and type contracts (`types.ts` or JSDoc typedefs).
3. **Implement** — Build components using `<script setup>` and Composition API with proper reactivity (`ref` for primitives, `reactive` for complex objects, `computed` for derived values).
4. **Validate**:
   - *TypeScript*: Run `vue-tsc --noEmit` and resolve all type discrepancies.
   - *Vanilla JS*: Verify JSDoc coverage on all public APIs and run ESLint with `eslint-plugin-jsdoc`.
5. **Optimize** — Minimize re-renders, optimize computed caching, implement virtual scrolling or lazy loading for heavy UI.
6. **Test** — Write component tests with Vue Test Utils and Vitest; fix any assertion or reactivity bugs until the suite is green.

---

## Reference Guide

Load detailed guidance based on context:

| Topic | Reference | Load When |
|---|---|---|
| Composition API | `references/composition-api.md` | `ref`, `reactive`, `computed`, `watch`, lifecycle hooks |
| Components (TS) | `references/components.md` | Props, emits, slots, `provide`/`inject` with TypeScript |
| Components (JSDoc) | `references/component-architecture.md` | Runtime props & JSDoc typed props/emits |
| State Management | `references/state-management.md` | Pinia stores, actions, getters, hydration |
| TypeScript Integration | `references/typescript.md` | Typing props, generic components, type safety |
| JSDoc Typing | `references/jsdoc-typing.md` | JSDoc types, `@typedef`, `@param`, type hints without TS compiler |
| Composables Patterns | `references/composables-patterns.md` | Reusable composables, cleanup logic, async state |
| Nuxt 3 & SSR | `references/nuxt.md` | SSR, file-based routing, `useFetch`, Fastify, hydration |
| Mobile & Hybrid | `references/mobile-hybrid.md` | Quasar, Capacitor, PWA, service workers |
| Build Tooling | `references/build-tooling.md` | Vite config, sourcemaps, optimization, bundling |
| Testing Patterns | `references/testing-patterns.md` | Vitest, component testing, mocking |

---

## Code Patterns

### Pattern A: Modern TypeScript (`lang="ts"`)

```vue
<script setup lang="ts">
import { ref, computed } from 'vue'

interface Props {
  initialCount?: number
  step?: number
}

const props = withDefaults(defineProps<Props>(), {
  initialCount: 0,
  step: 1
})

const emit = defineEmits<{
  (e: 'change', count: number): void
}>()

const count = ref(props.initialCount)
const doubled = computed(() => count.value * 2)

function increment() {
  count.value += props.step
  emit('change', count.value)
}
</script>

<template>
  <button @click="increment">
    Count: {{ count }} (doubled: {{ doubled }})
  </button>
</template>
```

### Pattern B: Vanilla JavaScript with JSDoc (`.mjs` / No TS)

```vue
<script setup>
/**
 * @typedef {Object} CounterProps
 * @property {number} [initialCount=0] - Initial starting counter
 * @property {number} [step=1] - Increment step
 */

/** @type {CounterProps} */
const props = defineProps({
  initialCount: { type: Number, default: 0 },
  step:         { type: Number, default: 1 }
})

/**
 * @typedef {Object} CounterEmits
 * @property {(count: number) => void} change - Emitted when count changes
 */
const emit = defineEmits(['change'])

const count = ref(props.initialCount)
const doubled = computed(() => count.value * 2)

function increment() {
  count.value += props.step
  emit('change', count.value)
}
</script>

<template>
  <button @click="increment">
    Count: {{ count }} (doubled: {{ doubled }})
  </button>
</template>
```

---

## Architectural Constraints

### MUST DO
- Always use Composition API (`<script setup>`) — never Options API.
- Respect project typing mode (TypeScript when configured, rich JSDoc when in vanilla JS mode).
- Use `ref()` for primitive values, `reactive()` for objects/records.
- Use `computed()` for derived calculations (pure, no side effects).
- Always clean up event listeners, timers, and external observers in `onUnmounted()` or `onScopeDispose()`.
- Use Pinia for modular, type-safe global state (never Vuex).

### MUST NOT DO
- Never mix Options API and Composition API in the same component.
- Never mutate props directly inside child components.
- Do not use `watch` when `computed` satisfies the requirement.
- Do not access DOM nodes before `onMounted()`.
- Do not introduce untyped `any` when TypeScript or JSDoc can provide exact contracts.
