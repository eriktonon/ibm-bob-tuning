---
name: elicitation
description: "Checkpoint de refinamento estruturado (socratic. Use quando: refinar decisoes arquiteturais."
user-invocable: true
---

# Skill Advanced Elicitation

Esta skill é um **checkpoint de refinamento** com catálogo de métodos estruturados para challengear decisões antes de avançar. É invocável em pausas naturais do fluxo (após arquitetura, após PRD, após SPEC) por qualquer agente ou pelo orquestrador.

Inspirada no `bmad-advanced-elicitation` do BMad Method, adaptada ao ecossistema PE-AGENTS-IBM.

---

## Quando Usar

- Após o SPEC estar congelado (challengear premissas antes de arquitetar)
- Após a arquitetura ser definida (red team antes de implementar)
- Quando uma decisão arquitetural for complexa ou controversa
- Antes do cutover de modernização (pre-mortem)
- Quando o orquestrador suspeitar de scope creep ou premissas implícitas

---

## Catálogo de Métodos

| Método | O que faz | Quando usar |
|---|---|---|
| **Socratic** | Faz perguntas progressivas para expor premissas implícitas | SPEC ou PRD com premissas não validadas |
| **Pre-mortem** | Assume que falhou e pergunta "por que falhou?" | Antes de implementar, cutover, ou modernização |
| **Red Team** | Designa um agente para atacar a decisão como adversário | Decisões arquiteturais complexas |
| **First Principles** | Decompõe até verdades irredutíveis e reconstrói | Quando a decisão depende de "best practices" não questionadas |
| **Subtraction** | Remove elementos e pergunta "ainda funciona?" | Combater scope creep, simplificar arquitetura |
| **Steelman** | Constrói o argumento mais forte contra a própria decisão | Antes de finalizar ADR |
| **Devil's Advocate** | Assume a posição oposta e defende | Quando há consenso rápido demais |
| **Edge Case Hunter** | Lista edge cases e casos extremos | Antes de definir ACs de stories |

---

## Procedimento

### Passo 1 — Selecione o método

Com base no contexto:
- Após SPEC → **Socratic** (validar premissas) ou **Subtraction** (combater scope creep)
- Após Arquitetura → **Red Team** ou **Pre-mortem**
- Antes de Cutover → **Pre-mortem**
- Decisão complexa → **First Principles** ou **Steelman**

### Passo 2 — Aplique o método

Execute o método como um sub-agente ou diretamente:

#### Socratic
1. Liste 3-5 premissas implícitas na decisão
2. Para cada premissa, pergunte: "Por que assumimos isso? Que evidência temos?"
3. Marque premissas como ✅ validada, ⚠️ incerta, ❌ falsa
4. Se ≥ 1 premissa ❌ falsa → revise a decisão

#### Pre-mortem
1. Imagine que a implementação foi para produção e **falhou catastroficamente**
2. Liste 5 possíveis causas da falha
3. Para cada causa, identifique: probabilidade (Alta/Média/Baixa) e mitigação
4. Adicione mitigações como constraints no SPEC ou ADR

#### Red Team
1. Designe um agente adversário (ex: `pe-agents-ibm-seguranca` para atacar arquitetura)
2. O adversário lista 5 vetores de ataque/fraqueza
3. Para cada vetor: impacto + contramedida
4. Incorpore contramedidas antes de avançar

### Passo 3 — Registre o resultado

Adicione o output da elicitação ao artefato relevante:
- SPEC → adicione premissas validadas em `Constraints`
- ADR → adicione seção `Elicitation Results`
- Story → refine ACs com base em edge cases encontrados

### Passo 4 — Indexe no project context bank (dual-persistence)

```bash
workspace rag knowledge-save "Elicitation" "pe-agents-ibm-orquestrador" "elicitation-<metodo>" "<achados da elicitação>" "<decisões refinadas>"
```

---

## Integração com o Orquestrador

O orquestrador deve invocar esta skill:
- Após o SPEC estar congelado (Socratic)
- Após a arquitetura ser definida (Red Team)
- Antes do cutover de modernização (Pre-mortem)
- Quando houver suspeita de scope creep (Subtraction)

> **Regra**: a elicitação não bloqueia o fluxo por padrão — mas se o Pre-mortem encontrar ≥ 2 causas de "Alta" probabilidade, o orquestrador deve pausar e pedir ao usuário para validar mitigações antes de avançar.
