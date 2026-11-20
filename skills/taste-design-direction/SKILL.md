---
name: taste-design-direction
description: "Direcao de arte, eliminacao de AI Slop. Use quando: desenhar UI autoral, auditar frontend ou validar."
user-invocable: true
---

# Skill Taste & Art Direction — Anti-AI-Slop & Web Craft

Esta skill unifica o protocolo empírico do **Taste Skill (`Leonxlnx/taste-skill`, estudo de 247 sites premiados)** com o nosso blueprint **`INDEX_DESIGN_AGENTS_ANTI_SLOP_BLUEPRINTS`** e as obras fundamentais indexadas no project context bank (**Don Norman**, **Steve Krug**, **Brad Frost**, **Claude Hopkins** e **Alex Hormozi**).

---

## 🔍 1. Grounding Prévio Obrigatório no project context bank

Antes de projetar ou codificar qualquer tela, recupere os critérios visuais, heurísticas de UX e especificidade de copywriting na base vetorial:

```bash
cargo run -- rag ground --agent pe-agents-ibm-frontend "INDEX_DESIGN_AGENTS_ANTI_SLOP_BLUEPRINTS Don Norman Steve Krug"
```

---

## 🎛️ 2. Direção de Arte & Seletores de Calibração (Before Coding)

Toda interface DEVE declarar explicitamente (ou seguir no briefing da Onda 1) três seletores de direção de arte e um **Inventário de Especificidade do Domínio**:

1. **`DESIGN_VARIANCE` (1 a 10)**:
   - `1–3`: Simétrico, utilitário e estrito (sistemas bancários, tabelas densas, formulários fiscais).
   - `4–7`: Assimetria controlada, Bento Grids editoriais, splits narrativos (Marketplaces, SaaS B2B moderno, E-commerce).
   - `8–10`: Composição autoral tipo pôster, tipografia monumental (Portfólios, Landing Pages de marca).
2. **`VISUAL_DENSITY` (1 a 10)**:
   - E-commerce Editorial / Luxo: `2–4/10` (amplo macro-whitespace `py-20` a `py-28`).
   - Marketplaces / Classificados (ex: OLX, Mercado Livre): `6–7/10` (filtros claros, grid escaneável de 4 colunas, badges de proteção de compra).
   - Fintech / DevTools / SRE Dashboards: `8–9/10` (alta densidade de dados, `tabular-nums`, atalhos de teclado `<kbd>`).
3. **`MOTION_INTENSITY` (1 a 10)**:
   - Feedback tátil obrigatório em controles (`active:scale-[0.98]`, `transition-all duration-150`) e transições suaves baseadas exclusivamente em `transform` e `opacity`, respeitando `@media (prefers-reduced-motion: reduce)`.
4. **Inventário de Especificidade (Claude Hopkins — *Scientific Advertising*)**:
   - Liste de **8 a 12 fatos concretos exclusivos daquele domínio** antes de criar os dados: cidades e bairros reais (`Pinheiros, SP`, `Savassi, BH`), preços plausíveis de mercado (`R$ 4.890,00`), modelos exatos (`iPhone 15 Pro 256GB Titânio Natural`), tempos e regras reais (`Garantia OLX Pay 48h`, `Frete Grátis Brasil`).
   - **PROIBIDO** usar `"Produto 1"`, `"Item Exemplo"`, `"John Doe"`, `"Acme Corp"`, `"Lorem Ipsum"` ou clichês de IA (`"Elevate"`, `"Seamless"`, `"Unleash"`, `"Next-Gen"`).

---

## 🚫 3. House Rules Anti-AI-Slop (Regras Determinísticas de Exclusão)

Estudos empíricos em 247 sites mostram que páginas fracas ($\le 6.5$) abusam de ornamentos artificiais que denunciam geração automática. Aplique estas regras sem exceção:

| Vício de IA (AI Slop / Tell) | Regra de Exclusão & Substituição de Bom Gosto |
| :--- | :--- |
| **Fileira de 3 Cards Idênticos com Ícone no Topo (`icon-cards`)** | Presente em 28% dos sites fracos vs. 1% dos premiados. **Substitua** por Bento Grid assimétrico, split narrativo, vitrine real de produtos com fotos ou lista tipográfica. |
| **Gradientes Violeta/Indigo Genéricos (`palette-purple`)** | **Banido** como padrão default (`from-indigo-500 to-purple-600`). Use a paleta autêntica da marca (ex: `#6E0AD6` + `#FF7A00` na OLX sobre fundo neutro `#F8FAFC`/`#FFFFFF`, ou `#0F62FE` no IBM Carbon). |
| **Sombras Pesadas e Glows (`shadow-xl`, neon)** | Sombras só existem no que realmente flutua (modais, drawers, dropdowns). Separe superfícies em repouso por **degrau de tom** (`#FFFFFF` sobre `#F7F6F3`/`#F8FAFC` no Light Mode; `#121215` sobre `#09090b` com `border-white/10` no Dark Mode) e borda fina estrutural (`border-slate-200`). |
| **Texto Corrido Cinza Fraco ou Micro-Texto (`< 12px`)** | Nunca deixe o texto principal lavado. Títulos e preços usam tinta cheia (`text-slate-900` / `#111111` no claro; `text-white` no escuro) com contraste $\ge 7:1$. Nada legível fica abaixo de `12px` (`text-xs`). |
| **Números e Preços Desalinhados** | Todo preço, KPI, contador, CEP ou tabela financeira DEVE usar `tabular-nums` (`font-variant-numeric: tabular-nums`). |
| **Tipografia sem Hierarquia** | Máximo de 2 famílias tipográficas. Títulos principais (`h1`/`h2`) com `tracking-tight` (`-0.02em` a `-0.03em`) e no máximo 2 linhas. Parágrafos com largura de leitura confortável (`max-w-[65ch]`). |
| **Caixas Cinzas ou Falta de Fotografia Real (`few-pictures`)** | Em páginas públicas, e-commerces e marketplaces, exija pelo menos **6 a 8 fotografias reais distintas em alta definição** (`https://images.unsplash.com/photo-...` contextualizadas ao item) com `aspect-[4/3]` ou `aspect-square`, `object-cover` e zoom sutil no hover (`group-hover:scale-105`). |
| **Emojis Soltos na Interface** | **Proibido** usar emojis (`🛒`, `🔒`, `✨`) como substitutos de ícones. Use exclusivamente ícones SVG vetoriais (`lucide-react` ou SVGs semânticos com `stroke-width` padronizado). |

---

## ⚙️ 4. Leis de Interatividade & Full-Output Enforcement (Norman, Krug & `output-skill`)

Uma interface bonita onde os controles não funcionam quebra a regra primordial de **Don Norman (*The Design of Everyday Things*)**: *todo Signifier visual promete uma Affordance real*.

1. **Proibição de Inputs e Botões Decorativos ("Fake UI")**:
   - Toda barra de busca (`<input type="text" ...>`) DEVE possuir `value={searchQuery}` e `onChange={(e) => setSearchQuery(e.target.value)}` filtrando os itens renderizados em tempo real.
   - Todo seletor de categoria, filtro de estado/UF ou ordenação DEVE alterar imediatamente a listagem visível e exibir estado ativo claro (`aria-pressed` ou classe visual distinta).
   - Todo botão de ação primária (`Comprar com Garantia`, `Adicionar ao Carrinho`, `Anunciar Grátis`, `Falar com Vendedor`) DEVE possuir handler `onClick` abrindo seu fluxo completo (Modal, Drawer ou mutação de API) e feedback imediato.
2. **Full-Output Enforcement (`output-skill`)**:
   - É terminantemente proibido truncar código com `// ...`, `// resto dos itens`, `// TODO` ou entregar modais vazios.

---

## 🧐 5. Rubrica de Inspeção Visual em 14 Passos (Awwwards Jury & Browser Check)

Compilar sem erros (`tsc` + `vite build`) significa apenas ausência de erro sintático. Antes de aprovar a entrega, avalie a interface renderizada no navegador (`1440px` desktop e `390px` mobile) segundo os **4 critérios ponderados**:

1. **Design & Direção de Arte (40%)**:
   - *Teste de 1 Segundo & Squint Test*: Ao piscar para a tela, existe 1 ponto focal claro e 2–3 massas visuais bem definidas?
   - *Alinhamento de Bordas*: Os blocos de texto e grids compartilham alinhamentos verticais limpos?
   - *Fotografia & Superfícies*: As fotos são reais, nítidas, sem distorção, e os cards têm respiro interno (`p-4` a `p-6`)?
2. **Usabilidade — Krug & Norman (30%)**:
   - *Orientação em 3 Segundos (*Don't Make Me Think*)*: Fica óbvio o que é a plataforma, para quem é e qual é a ação principal sem precisar rolar a página?
   - *Contraste & Legibilidade*: Todos os textos, badges e inputs têm contraste $\ge 4.5:1$?
   - *Fluxo Funcional Completo*: Busca, filtros por categoria/UF, abertura de detalhes, carrinho e checkout de 3 etapas funcionam do início ao fim?
3. **Criatividade & Identidade da Marca (20%)**:
   - *Teste de Cobrir a Logo*: Se cobrirmos o nome da marca no topo, a paleta, os componentes de confiança (ex: badge de Garantia, selo de verificação, calculadora de frete) e a linguagem visual ainda identificam claramente o produto?
4. **Conteúdo & Especificidade — Hopkins & Hormozi (10%)**:
   - Preços reais formatados em moeda local com `tabular-nums`, localizações verossímeis, avaliações realistas e proposta de valor clara (Equação de Valor de Hormozi no checkout: Garantia + Frete + Parcelamento sem juros).
