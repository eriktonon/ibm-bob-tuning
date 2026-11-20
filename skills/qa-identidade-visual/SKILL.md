---
name: qa-identidade-visual
description: Audita interfaces web segundo IBM Carbon Design System v11 e WCAG 2.1 AA (contraste, tokens, tipografia, alinhamento e hierarquia). Use quando revisar telas, componentes ou PRs visuais.
---

# QA de Identidade Visual (IBM Carbon v11 & WCAG 2.1 AA)

Checklist objetivo para revisar telas e componentes antes da entrega.

## Checklist de Auditoria
1. **Tokens & Paleta:** 0 cores `#HEX` arbitrarias fora do arquivo de tokens; aderencia consistente ao tema Carbon (`white`, `g10`, `g90` ou `g100`).
2. **Tipografia:** `IBM Plex Sans` para UI e `IBM Plex Mono` para metricas/IDs/codigo; hierarquia clara sem excesso de pesos ou tamanhos concorrentes.
3. **Geometria & Grid:** alinhamento rigoroso ao grid de 8px, bordas limpas (`1px` hairline) e espacamento respiravel sem ruido visual.
4. **Acessibilidade (WCAG 2.1 AA):**
   - Contraste >= 4.5:1 em textos e labels.
   - Todos os controles operaveis via teclado (`Tab`, `Enter`, `Escape`) com indicador de foco visivel (`outline: 2px solid var(--cds-focus)`).
   - Landmarks semanticos (`header`, `nav`, `main`, `footer`) e `aria-label` em botoes de icone.
5. **Estados Reais:** toda tela trata explicitamente estados `loading` (skeleton), `empty`, `error` acionavel e `success`.
6. **Zero Erros de Runtime:** sem erros no console do navegador, sem assets quebrados (404) e sem overflow horizontal em desktop (`1440px`) e mobile (`375px`).
