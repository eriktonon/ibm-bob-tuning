---
name: memlog
description: "Implementa memoria append-only por agente/skill: log. Use quando: registrar eventos granulares de."
user-invocable: true
---

# Skill Memlog — Memória Append-Only por Agente

Esta skill implementa um **log plano, cronológico, append-only** (`.memlog.md`) por agente/skill, complementando as 3 camadas de memória existentes (`session`/`repo`/`context.md`) com um log de eventos granular e atômico.

Inspirada no `memlog.py` do BMad Method, adaptada ao ecossistema PE-AGENTS-IBM.

---

## Quando Usar

- Registrar eventos granulares de trabalho por agente entre sessões
- Rastrear progresso de um agente específico sem poluir o `context.md`
- Manter "working memory" atômica e retomável
- Quando o `context.md` está ficando muito longo e o estado operacional precisa ser mais granular

---

## Princípios

| Princípio | O que significa |
|---|---|
| **Append-only** | Nunca edita ou remove entradas — apenas adiciona ao final |
| **Cronológico** | Entradas são ordenadas por timestamp |
| **Atômico** | Cada write é `fsync + rename` para evitar corrupção |
| **Por agente/skill** | Cada agente/skill tem seu próprio `.memlog.md` |
| **Estado inferido** | O estado é inferido lendo as últimas entradas, não via frontmatter mutável |

---

## Estrutura do Memlog

Arquivo: `.pe-agents-ibm/memlogs/{agent-name}.memlog.md`

```markdown
# Memlog — pe-agents-ibm-backend

## 2026-07-24T14:00:00Z | spec-created
Created SPEC.md for escola-notas app with 5 capabilities.

## 2026-07-24T14:30:00Z | task-received
[TASK] Implement CRUD for notas endpoint. Stack: Node.js + Fastify + PostgreSQL.

## 2026-07-24T15:00:00Z | implementation-complete
Created 6 endpoints: GET/POST/PUT/DELETE /api/notas, GET /api/notas/:id, GET /api/alunos/:id/notas.
Files: src/routes/notas.js, src/controllers/notas.js, src/models/notas.js.

## 2026-07-24T15:30:00Z | validation-pass
Tester validated all 6 ACs. Coverage: 85%. No CVEs found.
```

---

## Script de Runtime

`scripts/runtime/memlog.py` — operações atômicas de append:

```bash
# Append entry
python3 scripts/runtime/memlog.py append \
  --agent pe-agents-ibm-backend \
  --action "implementation-complete" \
  --content "Created 6 endpoints for notas."

# Read last N entries
python3 scripts/runtime/memlog.py tail \
  --agent pe-agents-ibm-backend \
  --n 5

# Read all
python3 scripts/runtime/memlog.py read \
  --agent pe-agents-ibm-backend

# Search
python3 scripts/runtime/memlog.py search \
  --agent pe-agents-ibm-backend \
  --query "notas"
```

---

## Relação com as 3 Camadas Existentes

| Camada | Propósito | Quando usar |
|---|---|---|
| `/memories/session/` | Estado transitório da sessão atual | Durante uma sessão |
| `/memories/repo/` | Fatos estáveis reutilizáveis | Decisões duráveis |
| `.pe-agents-ibm/context.md` | Estado auditável do projeto | Decisões, log de execução, validações |
| `.pe-agents-ibm/memlogs/{agent}.memlog.md` | **Working memory granular por agente** | Eventos atômicos, progresso, retomabilidade |

> **Regra**: não duplicar conteúdo entre camadas. Use memlog para eventos granulares que não justificam uma entrada no `context.md`.

---

## Integração com o Orquestrador

O orquestrador deve:
1. Ao iniciar uma delegação, fazer `memlog.py tail` do agente para retomar contexto
2. Ao concluir uma delegação, fazer `memlog.py append` com o resultado
3. Ao retomar uma sessão, ler os memlogs dos agentes ativos para reconstruir estado

---

## Integração com o Elasticsearch

Ao fazer `append`, o script também indexa no Elasticsearch:

```bash
workspace rag knowledge-save "Memlog" "<agent-name>" "<action>" "<content>" ""
```
