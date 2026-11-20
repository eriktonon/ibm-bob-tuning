---
name: ux-heuristicas
description: "Aplica heurísticas de usabilidade e design de produto. Use quando: criar interfaces."
user-invocable: true
---

# Skill Heurísticas de UX & Produto (Norman, Krug, Sprint)

Esta skill sintetiza e aplica na prática os princípios clássicos de design centrado no usuário e metodologias ágeis de produto consolidadas nos livros mestres indexados no project context bank:

1. **Steve Krug** — *Don't Make Me Think* (Eliminação de esforço mental e ruído visual)
2. **Don Norman** — *The Design of Everyday Things* (Affordances, Signifiers, Feedback e Mappings)
3. **Jake Knapp** — *Sprint: How to Solve Big Problems and Test New Ideas in Five Days* (Prototipação e validação rápida)

---

## 1. Princípios de Steve Krug (*Don't Make Me Think*)

- **Primeira Lei da Usabilidade**: Uma página ou tela deve ser autoevidente e óbvia. O usuário não deve ter que parar para pensar como usá-la.
- **Design para Escaneamento**: Usuários não leem tudo, eles "escaneiam". Utilize:
  - Hierarquia visual nítida (títulos, tamanhos de fonte, pesos contrastantes).
  - Quebra de blocos densos em listas e cartões (*chunking*).
  - Formatação visual que aproveita convenções conhecidas (ex: carrinhos de compras no topo direito, logos clicáveis para home).
- **Redução de Ruído Visual**: Elimine palavras e elementos visuais desnecessários. Se um elemento não ajuda na tomada de decisão, remova-o.

---

## 2. Princípios de Don Norman (*The Design of Everyday Things*)

- **Affordance & Signifiers**:
  - *Affordance*: A ação possível que um elemento suporta (ex: um botão pode ser clicado).
  - *Signifier*: A pista visual clara que comunica onde e como a ação deve acontecer (sombra sutil, borda, cor de destaque, cursor pointer).
- **Feedback Imediato**: Toda ação do usuário (clique, envio, drag, toggle) DEVE produzir resposta visual e/ou sonora em menos de 100ms (spinners, toasts, animações sutis, desativação de botão para evitar duplo clique).
- **Mapeamento Natural (Natural Mapping)**: A disposição dos controles na interface deve refletir intuitivamente a relação espacial ou lógica com os resultados desejados.
- **Prevenção e Tratamento de Erros**: Projete para evitar erros antes que ocorram (constraints em inputs, validação inline, mensagens de erro humanas e orientadas à ação corretiva).

---

## 3. Metodologia GV Sprint (Jake Knapp)

- **Foco no Problema Central**: Identifique o *Core User Flow* antes de construir telas secundárias.
- **Prototipagem Ágil**:
  1. **Map**: Desenhe a jornada do usuário em 5 passos.
  2. **Sketch**: Esboce as telas e componentes essenciais.
  3. **Decide**: Escolha a melhor hipótese sem paralisia por análise.
  4. **Prototype**: Construa um protótipo funcional ou MVP de alta fidelidade (Tailwind CSS / Lucide / React).
  5. **Test**: Teste empiricamente com métricas claras de sucesso.

---

## Checklist de Avaliação de Telas

Antes de aprovar qualquer entrega de frontend ou produto:
- [ ] O propósito da tela fica claro em menos de 3 segundos?
- [ ] O usuário consegue escanear os pontos focais sem ler parágrafos densos?
- [ ] Os botões e links possuem signifiers evidentes (não se parecem com texto puro)?
- [ ] Há feedback visual para ações de carregamento, sucesso ou erro?
- [ ] O fluxo principal atende ao objetivo sem bifurcações desnecessárias?
