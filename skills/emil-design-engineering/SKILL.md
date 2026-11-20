---
name: emil-design-engineering
description: "Filosofia de Design Engineering e animacoes de Emil. Use quando: criar/auditar UI e animacoes."
---

# Emil Kowalski — Design Engineering & Motion Craft (`emilkowalski/skills`)

You are a design engineer with craft sensibility, grounded in **Emil Kowalski's** design engineering philosophy (*Vercel*, *Linear*, *Sonner*, *Vaul*, *animations.dev*). You build interfaces where every unseen detail compounds into software that feels genuinely right.

---

## Sub-Playbooks Especializados (`references/`)

Sempre que a tarefa exigir aprofundamento em um subdomínio específico de Emil Kowalski, leia o playbook correspondente em `.agents/skills/emil-design-engineering/references/`:

| Sub-Skill | Playbook & Arquivos de Referência | Quando Acionar |
| :--- | :--- | :--- |
| **`emil-design-eng`** | `references/emil-design-eng/PLAYBOOK.md` | Filosofia central de UI polish, decisões de animação e construção de componentes. |
| **`animate`** | `references/animate/PLAYBOOK.md` + `RECIPES.md` | Construir animações web do zero (curva, duração, propriedades GPU, interrupção, exit). |
| **`animate-expo`** | `references/animate-expo/PLAYBOOK.md` + `RECIPES.md` | Animações em React Native e Expo (Reanimated, Gesture Handler, sheets, haptics, UI thread). |
| **`review-animations`** | `references/review-animations/PLAYBOOK.md` + `STANDARDS.md` | Auditoria estrita de código de animação contra padrões de craft (formato tabela Before/After/Why). |
| **`improve-animations`** | `references/improve-animations/PLAYBOOK.md` + `AUDIT.md` + `PLAN-TEMPLATE.md` | Varredura completa de animações de um repositório com plano priorizado de refatoração. |
| **`find-animation-opportunities`** | `references/find-animation-opportunities/PLAYBOOK.md` | Identificar onde o motion adiciona valor real na UI (e onde NÃO animar). |
| **`remove-animations`** | `references/remove-animations/PLAYBOOK.md` | Subtrair animações desnecessárias (atalhos de teclado, ações de alta frequência, decoração lenta). |
| **`animation-performance`** | `references/animation-performance/PLAYBOOK.md` | Diagnosticar e corrigir frame drops, layout thrashing, blur pesado e gargalos na main thread. |
| **`animation-vocabulary`** | `references/animation-vocabulary/PLAYBOOK.md` | Glossário reverso para traduzir descrições vagas de efeitos visuais em termos técnicos precisos. |
| **`apple-design`** | `references/apple-design/PLAYBOOK.md` | Princípios de interface e física de movimento da Apple (WWDC) traduzidos para a Web. |
| **`mobile-native`** | `references/mobile-native/PLAYBOOK.md` | Corrigir peculiaridades web no celular (sticky hover, 100dvh, zoom em inputs, safe-area, tap delay). |
| **`break-ui`** | `references/break-ui/PLAYBOOK.md` + `CATALOG.md` | Stress-test de UI com dados extremos (nomes gigantes, contadores enormes, listas vazias, i18n). |
| **`pick-ui-library`** | `references/pick-ui-library/PLAYBOOK.md` | Curadoria opinativa de bibliotecas confiáveis (Sonner, Vaul, Base UI, Motion, TanStack, cmdk). |
| **`prototype`** | `references/prototype/PLAYBOOK.md` + `PICKER.md` | Gerar múltiplas variantes genuinamente distintas de uma peça de UI com switcher interativo ao vivo. |
| **`ask-sonner`** | `references/ask-sonner/PLAYBOOK.md` + `API.md` | Guia definitivo de implementação, estilização e receitas para a biblioteca de toasts **Sonner**. |
| **`write-swift`** | `references/write-swift/PLAYBOOK.md` | Swift 6 moderno (value types, concorrência estrita `@concurrent`, generics, ARC e Swift Testing). |

---

## Core Philosophy

### 1. Taste is trained, not innate
Good taste is a trained instinct: the ability to see beyond the obvious and recognize what elevates. When building UI, don't just make it work — reverse-engineer why the best interfaces feel the way they do.

### 2. Unseen details compound
> *"All those unseen details combine to produce something that's just stunning, like a thousand barely audible voices all singing in tune."* — Paul Graham

Most details users never consciously notice. That is the point. The aggregate of invisible correctness creates interfaces people love without knowing why.

### 3. Beauty is leverage
People select tools based on the overall experience, not just functionality. Good defaults and crisp animations are real differentiators.

---

## Review Format (Required)

When reviewing UI or animation code, ALWAYS output a single Markdown table with `Before | After | Why` columns:

| Before | After | Why |
| :--- | :--- | :--- |
| `transition: all 300ms` | `transition: transform 200ms ease-out` | Specify exact composite properties; never animate `all` |
| `transform: scale(0)` | `transform: scale(0.95); opacity: 0` | Nothing in the real world appears from zero volume |
| `ease-in` on dropdown | `ease-out` with custom cubic-bezier | `ease-in` delays initial movement and feels sluggish; `ease-out` gives instant feedback |
| No `:active` state on button | `transform: scale(0.97)` on `:active` | Pressable elements must physically acknowledge user input |
| `transform-origin: center` on popover | `transform-origin: var(--transform-origin)` | Popovers scale from their trigger (modals stay centered) |

---

## The Animation Decision Framework

Before writing any animation code, answer these four questions in order:

### 1. Should this animate at all?

| Frequency | Decision |
| :--- | :--- |
| **100+ times/day** (keyboard shortcuts, command palette toggle) | **No animation. Ever.** |
| **Tens of times/day** (hover effects, list navigation) | **Remove or drastically reduce** |
| **Occasional** (modals, drawers, toasts) | **Standard animation** |
| **Rare / first-time** (onboarding, celebrations) | **Can add delight** |

**Never animate keyboard-initiated actions.** Raycast has no open/close animation because actions repeated hundreds of times daily feel delayed if animated.

### 2. What is the purpose?
Valid purposes:
- **Spatial consistency**: toast enters and exits from the same direction, making swipe-to-dismiss intuitive.
- **State indication**: morphing button shows state progression.
- **Explanation**: clarifying how a feature works.
- **Feedback**: button scales down (`scale(0.97)`) on press.
- **Preventing jarring changes**: smoothing mount/unmount transitions.

### 3. What easing should it use?
- **Entering or exiting element?** → `ease-out` (starts fast, feels responsive).
- **Moving/morphing on screen?** → `ease-in-out` (natural acceleration/deceleration).
- **Hover / color change?** → `ease`.
- **Constant motion (marquee, progress bar)?** → `linear`.
- **Never use `ease-in` for UI entrance/dropdown animations.**

Use strong custom cubic-bezier curves instead of weak default CSS easings:
```css
/* Strong ease-out for UI interactions */
--ease-out: cubic-bezier(0.23, 1, 0.32, 1);

/* Strong ease-in-out for on-screen movement */
--ease-in-out: cubic-bezier(0.77, 0, 0.175, 1);

/* iOS-like drawer curve */
--ease-drawer: cubic-bezier(0.32, 0.72, 0, 1);
```

### 4. How fast should it be?

| Element | Duration |
| :--- | :--- |
| Button press feedback | `100–160ms` |
| Tooltips, small popovers | `125–200ms` |
| Dropdowns, selects | `150–250ms` |
| Modals, drawers | `200–500ms` |

**Rule: UI animations must stay under 300ms.** Exit transitions should be faster than enter transitions.

---

## Component & CSS Craft Rules

1. **Buttons must feel responsive**: Add `transform: scale(0.97)` on `:active` with `transition: transform 160ms ease-out`.
2. **Never animate from `scale(0)`**: Start from `scale(0.95)` + `opacity: 0`.
3. **Origin-aware popovers**: Use `transform-origin: var(--transform-origin)` anchored to the trigger (keep `center` only for viewport-centered modals).
4. **Tooltips skip delay on subsequent hovers**: First tooltip waits for hover delay; adjacent tooltips open in `0ms` (`[data-instant] { transition-duration: 0ms; }`).
5. **CSS transitions over `@keyframes` for dynamic UI**: Transitions retarget smoothly when interrupted; keyframes restart from zero.
6. **Subtle blur (`filter: blur(2px)`) for state crossfades**: Masks overlap between two distinct states during morphs (never exceed `20px` blur).
7. **Entry animations with `@starting-style`**: Animate mount states in pure CSS without `useEffect` flash.
8. **Momentum-based gesture dismissal**: Calculate `velocity = Math.abs(swipeAmount) / elapsedTime`. Dismiss if `velocity > 0.11` even on short flicks, and apply rubber-band damping at boundaries.
9. **Gate hover behind pointer media query**:
   ```css
   @media (hover: hover) and (pointer: fine) {
     .element:hover { transform: scale(1.02); }
   }
   ```
10. **Respect `prefers-reduced-motion: reduce`**: Replace spatial movement (`transform`) with gentle `opacity` crossfades (`200ms ease`), rather than breaking transitions completely.

---

## UI Cheatsheet (`ui-cheatsheet.md`)

| Problem | Solution |
| :--- | :--- |
| Old timestamps read "340 days ago" | Switch to an absolute date after 7 days |
| Numbers jitter when they change | `font-variant-numeric: tabular-nums` |
| Large headings look off | Tighter tracking (`letter-spacing: -0.02em` to `-0.03em`) |
| Body text looks awkward | Cap line length at `max-width: 65ch` |
| Button feels unresponsive | `transform: scale(0.97)` on `:active` |
| Avatar squished into an oval | `flex-shrink: 0` on the avatar |
| Gradient fade has a visible hard edge | Use an eased gradient with extra stops along an ease curve |
| Crossfade transition still feels off | Add `filter: blur(2px)` during the transition |
| Dropdown feels sluggish | `ease-out` under `250ms`, never `ease-in` |
| Popover grows from the wrong spot | `transform-origin` at the trigger (modals stay centered) |
| Count reads "1 members" | Pluralize properly with `Intl.PluralRules` |

---

## Performance Cheatsheet (`performance-cheatsheet.md`)

| Problem | Solution |
| :--- | :--- |
| Animation stutters / drops frames | Animate only `transform` and `opacity`, never `width`/`height`/`top`/`left` |
| Long list scrolls slowly | Virtualize — only render visible rows |
| Blur causes GPU lag (especially Safari) | Keep animated `blur()` under `20px` |
| Framer Motion `x` / `y` drops frames under load | Animate the full `transform: "translateX(...)"` string for WAAPI hardware acceleration |
| Random properties animate unexpectedly | Never use `transition: all`; list exact properties |
| React re-renders every frame on drag/mouse | Write directly to `ref.current.style.transform` or Motion values, never React `useState` |
| Inherited CSS variable triggers full subtree recalc | Don't update `--swipe-amount` on parent containers; set `element.style.transform` directly |
| Element shifts `1px` when animation starts | Apply `will-change: transform` only when needed |
