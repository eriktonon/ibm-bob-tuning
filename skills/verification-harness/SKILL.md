---
name: verification-harness
description: "Validacao deterministica de compilacao. Use quando: validar codigo antes da entrega e impedir builds."
user-invocable: true
---

# Skill Verification Harness — Validação Determinística & Auto-Correção

Esta skill define o protocolo rigoroso do **Verification Harness** integrado nativamente ao Rust (`src/modules/verification_harness.rs`) e ao pipeline do J.A.R.V.I.S. O objetivo é eliminar alucinações de código, garantindo que nenhum projeto, arquivo ou microsserviço seja dado como concluído sem build verde comprovado por compilador ou interpretador real.

---

## Princípios Centrais

1. **Deterministic Syntax-First Verification**: Compiladores e linters são a fonte da verdade absoluta, não o auto-julgamento do LLM.
2. **Strict Loop Boundary**: Até 3 ciclos iterativos de auto-correção (Diff Patcher). Se o build não passar após 3 tentativas, o pipeline entra em estado `BLOCKED` e alerta o operador humano.
3. **Bloqueio de Persistência**: É TERMINANTEMENTE PROIBIDO salvar código com erro de build no project context bank ou marcar a story como `complete`.

---

## Matriz de Verificação por Tecnologia

| Tecnologia / Artefato | Comando Determinístico | Critério de Sucesso |
|---|---|---|
| **Rust Backend / CLI** | `cargo check` ou `cargo test` | Exit code 0, 0 erros no stderr |
| **Node.js / React** | `npm run build` ou `npx tsc --noEmit` | Exit code 0, build gerado sem erros |
| **Docker / Microserviço** | `docker compose config -q` & `docker compose build` | Sintaxe válida e imagens compiladas |
| **Go Backend** | `go build ./...` ou `go vet ./...` | Exit code 0 sem compilação pendente |
| **Python Backend** | `python -m py_compile <file>` ou `pytest` | Sintaxe válida sem SyntaxError |

---

## Fluxo de Auto-Correção (Self-Correction Loop)

```
[Geração de Código]
        ↓
[Execução do Harness (Cargo/NPM/Docker)]
        ↓
  Passou no Build?
    ├── SIM → Libera Handoff, salva no project context bank e finaliza
    └── NÃO → [Captura stderr/stdout completo]
                    ↓
              [Injeta Erro no Prompt de Correção do LLM]
                    ↓
              [Aplica Diff Patcher no arquivo afetado]
                    ↓
              [Re-executa Build (Máximo: 3 tentativas)]
```

---

## Regras de Execução para os Agentes

- O **Orquestrador** (`pe-agents-ibm-orquestrador`) deve invocar o Verification Harness antes de encerrar qualquer onda de entrega.
- O **Tester** (`pe-agents-ibm-tester`) deve validar sintaxe de Docker Compose e imagens locais além dos testes unitários.
- O agente implementador (`backend`, `frontend`, `sre`) deve receber os diagnósticos de compilador sem ruído e corrigir pontualmente os arquivos com patches atômicos.
