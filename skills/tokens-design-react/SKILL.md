---
name: tokens-design-react
description: Implementa tokens de design IBM Carbon Design System v11 em projetos React/Next.js/TypeScript. Use quando configurar temas, paleta, tipografia IBM Plex, espacamento e variaveis CSS/Sass.
---

# Design Tokens — IBM Carbon Design System v11

Padroniza o uso de tokens oficiais do **IBM Carbon Design System v11** em aplicacoes React e Next.js, eliminando cores e espacamentos hardcoded.

## Tokens Fundamentais (Carbon v11)

- **Temas suportados:** `white`, `g10` (Light), `g90`, `g100` (Dark).
- **Cores primarias e neutras:**
  - `--cds-interactive`: `#0f62fe` (IBM Blue 60)
  - `--cds-background` (g100): `#161616` | (g10): `#f4f4f4` | (white): `#ffffff`
  - `--cds-layer-01` (g100): `#262626` | (g10): `#ffffff`
  - `--cds-border-subtle-01`: `#393939` (g100) | `#e0e0e0` (g10)
  - `--cds-text-primary`: `#f4f4f4` (g100) | `#161616` (g10)
  - `--cds-support-error`: `#da1e28` | `--cds-support-success`: `#24a148` | `--cds-support-warning`: `#f1c21b` | `--cds-support-info`: `#0043ce`
- **Tipografia:** `'IBM Plex Sans', sans-serif` (UI) e `'IBM Plex Mono', monospace` (codigo/dados tabulares).
- **Escala de espacamento (2x Grid — multiplos de 8px):** `01: 2px`, `02: 4px`, `03: 8px`, `04: 12px`, `05: 16px`, `06: 24px`, `07: 32px`, `08: 40px`, `09: 48px`.

## Regras de Implementacao
1. Prefira `@carbon/react` e `@carbon/styles` quando disponivel; em projetos com Tailwind CSS, mapeie os tokens `--cds-*` em variaveis CSS no `:root` e consuma via `var(--cds-*)`.
2. Nunca escreva valores `#HEX` soltos dentro de componentes de feature; centralize 100% na camada de tokens.
3. Preserve contraste minimo WCAG 2.1 AA (`>= 4.5:1` texto normal, `>= 3:1` texto grande e componentes interativos).
