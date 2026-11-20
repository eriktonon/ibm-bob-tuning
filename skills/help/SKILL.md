---
name: help
description: "Recomenda o proximo passo com base nos artefatos no. Use quando: o usuario perguntar qual o proximo."
user-invocable: true
---

# Skill Help — Navegação de Próximo Passo

Esta skill recomenda o **próximo passo** com base nos artefatos já produzidos no disco (SPEC.md, context.md, migrations, código, Dockerfile, etc.). Lê um catálogo de fases e dependências e sugere qual skill ou agente acionar a seguir.

Inspirada no `bmad-help` do BMad Method, adaptada ao ecossistema PE-AGENTS-IBM.

---

## Quando Usar

- O usuário pergunta "o que faço agora?" ou "qual o próximo passo?"
- O orquestrador precisa decidir qual agente acionar mas não há fluxo claro
- Após uma entrega, para saber o que vem a seguir
- No início de uma sessão, para retomar de onde parou

---

## Catálogo de Fases

| Fase | Artefato esperado | Skill/Agente | Precedido por |
|---|---|---|---|
| 0 — Intent | `.pe-agents-ibm/SPEC.md` | `spec` | — |
| 0 — Elicitação | Resultado no SPEC/ADR | `elicitation` | SPEC |
| 1 — Arquitetura | ADR / Architecture Spine | `pe-agents-ibm-arquitetura` | SPEC |
| 1 — Decisão de Linguagem | Seção no context.md | `decisao-linguagem` | Arquitetura |
| 1 — Contrato de API | `docs/contracts/api.md` | `contrato-api` | Arquitetura + Linguagem |
| 2 — Banco de Dados | `migrations/*.sql` | `pe-agents-ibm-banco-dados` | Contrato API |
| 2 — Stories | `.pe-agents-ibm/stories/*.md` | `story` | Contrato API |
| 3 — Backend | Código backend | `pe-agents-ibm-backend` | Banco + Stories |
| 3 — Frontend | Código frontend | `pe-agents-ibm-frontend` | Contrato API + Backend |
| 4 — Segurança | Relatório de audit | `pe-agents-ibm-seguranca` | Backend + Frontend |
| 4 — Testes | Relatório de validação | `pe-agents-ibm-tester` | Backend + Frontend |
| 5 — SRE | Dockerfile + k8s manifests | `pe-agents-ibm-sre` | App builda |
| 5 — Documentação | README + ADRs | `pe-agents-ibm-documentacao` | Backend + Frontend |
| 6 — Modernização | Plano de migração | `pe-agents-ibm-modernizacao` | Contexto de legado |
| 6 — Engenharia Reversa (REA) | Evidence Ledger / Spec reconstruída | `reverse-engineer-anything` / `pe-agents-ibm-reverse-engineer` | Binário, pacote, ASAR, APK ou captura |
| 6 — Git Workflow | Branch + commit + merge | `git-workflow` | Entrega completa |

---

## Procedimento

### Passo 1 — Inspecione artefatos no disco

Verifique a existência de:
```bash
ls .pe-agents-ibm/SPEC.md 2>/dev/null
ls .pe-agents-ibm/context.md 2>/dev/null
ls docs/contracts/ 2>/dev/null
ls migrations/ 2>/dev/null
ls .pe-agents-ibm/stories/ 2>/dev/null
ls src/ || ls app/ || ls server/ 2>/dev/null
ls Dockerfile 2>/dev/null
ls k8s/ 2>/dev/null
```

### Passo 2 — Consulte a memória operacional (memlog + project context bank)

```bash
workspace memlog tail --n 5
workspace story status
workspace rag search-all "<projeto> proximos passos"
```

Isto retorna as últimas ações registradas, o estado dos stories e as decisões indexadas, indicando onde o projeto parou.

### Passo 3 — Cruze com o catálogo

Compare artefatos existentes com o catálogo de fases:
- Se o último artefato da fase N existe mas o da fase N+1 não → recomende a skill/agente da fase N+1
- Se um artefato está incompleto (ex: SPEC existe mas está vazio) → recomende completar a fase atual
- Se todos os artefatos até a fase 5 existem → recomende `git-workflow` para finalizar

### Passo 4 — Apresente a recomendação

Formato de resposta:
```
## Estado atual do projeto

Artefatos encontrados:
- ✅ .pe-agents-ibm/SPEC.md (fase 0)
- ✅ .pe-agents-ibm/context.md (decisões registradas)
- ❌ docs/contracts/ (fase 1 — pendente)
- ❌ migrations/ (fase 2 — pendente)

## Próximo passo recomendado

→ Invoque a skill `contrato-api` para definir o contrato de API.
  Precedido por: SPEC + Arquitetura (✅ já existem)

## Ou, se preferir:
- `decisao-linguagem` se a linguagem ainda não foi definida
- `pe-agents-ibm-banco-dados` se o contrato já estiver congelado
```

---

## Regras

- **Nunca pule fases** — se o SPEC não existe, recomende `spec` antes de arquitetura
- **Respeite dependências** — não recomende backend antes de contrato de API
- **Considere o context bank** — se houver uma seção `## O Que Já Foi Tentado` relevante, mencione
- **Seja conciso** — máximo 1 recomendação primária + 2 alternativas
- **Indexe a recomendação** no Elasticsearch para rastreabilidade
