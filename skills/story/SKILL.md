---
name: story
description: "Gera stories estruturados com Acceptance Criteria. Use quando: decompor SPEC em stories executaveis."
user-invocable: true
---

# Skill Story Template

Esta skill gera **stories estruturados** a partir do SPEC ou de epics, com Acceptance Criteria no formato Given/When/Then, Tasks numeradas e um Dev Agent Record para rastreabilidade. Conecta as `Capabilities` do SPEC com a validação do `pe-agents-ibm-tester` via ACs.

Inspirado no Story Template do BMad Method, adaptado ao ecossistema PE-AGENTS-IBM.

---

## Quando Usar

- Após o SPEC estar congelado (skill `spec`)
- Quando precisar decompor capabilities em unidades executáveis de dev
- Antes de delegar para o `pe-agents-ibm-backend` ou `pe-agents-ibm-frontend`
- Para rastrear qual agente implementou qual story e o status

---

## Template de Story

Crie um arquivo por story em `.pe-agents-ibm/stories/`:

```markdown
# STORY-[N]: [Título curto]

## Epic
[Epic pai ou Capability do SPEC]

## User Journey
Como [persona], quero [ação], para [valor de negócio]

## Acceptance Criteria

### AC-1: [Descrição curta]
**Given** [contexto inicial]
**When** [ação executada]
**Then** [resultado esperado]

### AC-2: [Descrição curta]
**Given** ...
**When** ...
**Then** ...

## Tasks
- [ ] T1: [tarefa técnica 1]
- [ ] T2: [tarefa técnica 2]
- [ ] T3: [tarefa técnica N]

## Dev Agent Record
| Data | Agente | Ação | Status |
|---|---|---|---|
| | | | |

## Status
draft | ready-for-dev | in-progress | review | complete | blocked

## Dependencies
- [Story-X, Skill-Y, ou "nenhuma"]
```

---

## Procedimento

### Passo 1 — Decompor SPEC em epics

Leia `.pe-agents-ibm/SPEC.md` e agrupe `Capabilities` em epics lógicos (2-5 stories por epic).

### Passo 2 — Gerar stories

Para cada epic, crie stories seguindo o template acima. Cada story deve:
- Ter **2-5 ACs** no formato Given/When/Then
- Ter **Tasks** técnicas acionáveis
- Referenciar o **Epic** e a **Capability** do SPEC
- Ter **Status** inicial `draft`

### Passo 3 — Validar gates de "Ready for Development"

Antes de mudar status para `ready-for-dev`, verifique:
- [ ] SPEC congelado e referenciado
- [ ] ACs são testáveis pelo `pe-agents-ibm-tester`
- [ ] Dependencies resolvidas
- [ ] Tasks são acionáveis por um único agente

### Passo 4 — Indexar no project context bank (dual-persistence)

```bash
workspace rag knowledge-save "Story Template" "pe-agents-ibm-orquestrador" "story-created" "<resumo do story>" "<ACs e tasks>"
```

### Passo 5 — Conectar com o Tester

Ao delegar para backend/frontend, passe as **ACs do story** como critério de validação. O `pe-agents-ibm-tester` deve validar cada AC individualmente.

---

## State Machine de Status

```
draft → ready-for-dev → in-progress → review → complete
                                          ↘ blocked
```

| Transição | Quem dispara |
|---|---|
| draft → ready-for-dev | Esta skill (gates validados) |
| ready-for-dev → in-progress | Agente backend/frontend ao começar |
| in-progress → review | Agente ao concluir implementação |
| review → complete | Tester valida todos os ACs (PASS) |
| review → blocked | Tester retorna FAIL_BLOCKED |

> **Regra**: o status fica no frontmatter do story, não no context bank — é o estado local de cada unidade de trabalho.
