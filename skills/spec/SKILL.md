---
name: spec
description: "Destila o intent do usuário em um SPEC.md canônico de 5. Use quando: iniciar nova aplicacao."
user-invocable: true
---

# Skill SPEC Kernel

Esta skill destila qualquer input do usuário (brain dump, PRD, transcript, pedido informal) em um **`SPEC.md`** — o contrato canônico de 5 campos que todo downstream consome. É a primeira formalização do intent, **antes** do `contrato-api` e da arquitetura.

Inspirado no SPEC Kernel do BMad Method (`bmad-code-org/BMAD-METHOD`), adaptado ao ecossistema PE-AGENTS-IBM.

---

## Quando Usar

- O usuário pede para criar uma aplicação nova e o intent ainda é informal
- O usuário pede para modernizar um sistema e o escopo precisa ser delimitado
- Antes de invocar `pe-agents-ibm-arquitetura` quando o pedido for vago ou ambíguo
- Sempre que houver risco de scope creep ou expectativas implícitas não registradas

---

## Os 5 Campos do SPEC

| Campo | O que é | Exemplo |
|---|---|---|
| **Why** | O problema de negócio que motiva a existência do sistema | "Escolas precisam lançar notas digitalmente para reduzir erro manual" |
| **Capabilities** | O que o sistema deve fazer (verbos de capacidade, não features) | "Registrar alunos, lançar notas por bimestre, gerar boletim PDF" |
| **Constraints** | Limites técnicos, operacionais e de compliance | "PostgreSQL, até 50 escolas, LGPD, deploy em AKS" |
| **Non-goals** | O que explicitamente NÃO está no escopo | "Não inclui gestão financeira, não inclui portal dos pais nesta fase" |
| **Success signal** | Como saber que deu certo (mensurável) | "Professor lança nota em < 30s; boletim gerado em < 5s; zero erro de cálculo" |

---

## Procedimento

### Passo 1 — Colete o intent

Se o input do usuário for informal, faça perguntas curtas para preencher os 5 campos. Se já houver PRD ou descrição estruturada, extraia diretamente.

### Passo 2 — Gere o SPEC.md

Crie o arquivo `.pe-agents-ibm/SPEC.md` no workspace do projeto-alvo:

```markdown
# SPEC — [Nome do Sistema]

## Why
[1-3 parágrafos sobre o problema de negócio]

## Capabilities
- [Capability 1: verbo + objeto]
- [Capability 2]
- [Capability N]

## Constraints
- [Constraint técnica]
- [Constraint operacional]
- [Constraint de compliance]

## Non-goals
- [Non-goal 1]
- [Non-goal N]

## Success signal
- [Métrica 1]
- [Métrica N]
```

### Passo 3 — Valide com o usuário

Apresente o SPEC.md e pergunte: "Este SPEC captura corretamente o seu intent? Há algo a ajustar?"

### Passo 4 — Indexe no project context bank (dual-persistence)

```bash
workspace rag knowledge-save "SPEC Kernel" "pe-agents-ibm-orquestrador" "spec-created" "<resumo do SPEC>" "<decisões do SPEC>"
```

### Passo 5 — Delegue para downstream

Com o SPEC congelado, delegue para:
- `pe-agents-ibm-arquitetura` — para desenhar a arquitetura alinhada ao SPEC
- `pe-agents-ibm-banco-dados` — para modelar dados dentro das constraints
- `pe-agents-ibm-backend` — para implementar as capabilities
- `pe-agents-ibm-frontend` — para construir a UI das capabilities

> **Regra**: todo agente downstream deve receber o SPEC.md como parte do `CONTEXT_BRIEF`. O SPEC é o contrato de intent — não reabra discussões já congeladas nele.

---

## Contrato com skills downstream

| Skill | O que consome do SPEC |
|---|---|
| `contrato-api` | `Capabilities` → endpoints; `Constraints` → limites de API |
| `decisao-arquitetura` | `Constraints` → estilo arquitetural; `Non-goals` → evitar over-engineering |
| `decisao-linguagem` | `Constraints` → linguagem; `Success signal` → requisitos de performance |
| `story` | `Capabilities` → epics → stories com ACs |
| `scaffold-clean-arch` | `Capabilities` → módulos; `Constraints` → stack |

---

## Anti-padrões

- ❌ NÃO pular o SPEC e ir direto para arquitetura quando o intent for vago
- ❌ NÃO incluir features em `Capabilities` que pertencem a `Non-goals`
- ❌ NÃO fazer `Success signal` subjetivo ("ficar bom") — deve ser mensurável
- ❌ NÃO editar o SPEC depois de congelado sem registrar a mudança no context bank
