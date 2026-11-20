---
name: roadmap-produto-tecnico
description: "Cria e mantem roadmaps tecnicos alinhados a arquitetura. Use quando: planejar evolucao tecnica."
user-invocable: true
---

# Roadmap de Produto — Tecnico

## Quando Usar

- Ao planejar evolucao da arquitetura ou stack tecnologica
- Quando for necessario endividar ou quitar divida tecnica
- Antes de iniciar migracoes de banco, framework ou cloud provider
- Ao definir milestones de performance, seguranca ou escalabilidade
- Quando o time de engenharia precisar de visibilidade de trabalho tecnico
- Para alinhar prioridades tecnicas com objetivos de negocio

## Procedimento

### Passo 1 — Diagnostico Tecnico Atual

Levante o estado atual do projeto respondendo:

| Area | Perguntas de Diagnostico |
|---|---|
| Arquitetura | Qual a arquitetura atual? Monolito, microservicos, serverless? Ha acoplamentos indesejados? |
| Stack | Quais linguagens, frameworks e versoes estao em uso? Ha componentes legados ou EOL? |
| Performance | Quais sao os gargalos atuais? Latencia, throughput, memoria, CPU? |
| Seguranca | Ha vulnerabilidades conhecidas? Dependencias desatualizadas? Compliance pendente? |
| Infraestrutura | Qual o ambiente de deploy? Ha automacao suficiente? Custos estao controlados? |
| Monitoramento | Ha observabilidade adequada? Logs, metricas, tracing, alerting? |
| Divida Tecnica | Quais atalhos foram tomados? Ha codigo sem testes, sem documentacao, duplicado? |
| Capacidade do Time | Quantos devs? Quais especialidades? Ha gargalo de conhecimento (bus factor)? |

### Passo 2 — Definicao de Temas e Iniciativas

Agrupe o trabalho tecnico em temas estrategicos:

```
Tema: Modernizacao de Stack
  Iniciativa 1: Migracao de Node.js 16 para 20 LTS
  Iniciativa 2: Substituicao de biblioteca de UI legada por novo design system
  Iniciativa 3: Adocao de TypeScript strict mode em todo o codebase

Tema: Performance e Escalabilidade
  Iniciativa 1: Implementacao de cache distribuido (Redis)
  Iniciativa 2: Otimizacao de queries com indexacao e paginacao
  Iniciativa 3: Migracao de imagens para CDN com formatos modernos

Tema: Seguranca e Compliance
  Iniciativa 1: Auditoria OWASP Top 10 e correcao de vulnerabilidades
  Iniciativa 2: Implementacao de MFA em todas as contas de servico
  Iniciativa 3: Criptografia em repouso para dados sensiveis

Tema: Developer Experience
  Iniciativa 1: Reducao de tempo de build e CI/CD
  Iniciativa 2: Padronizacao de ambientes locais com Docker Compose
  Iniciativa 3: Documentacao tecnica automatizada (OpenAPI, ADRs)
```

### Passo 3 — Priorizacao com RICE Tecnico

Aplique a formula RICE adaptada para contexto tecnico:

```
RICE = (Reach * Impact * Confidence) / Effort

Reach: Quantos devs, sistemas ou usuarios serao afetados positivamente?
Impact: Quanto melhora a metrica alvo? (3 = massivo, 2 = alto, 1 = medio, 0.5 = baixo)
Confidence: Quao confiante voce esta nos dados? (100% = alta, 80% = media, 50% = baixa)
Effort: Quantas semanas de trabalho de um dev senior?
```

| Iniciativa | Reach | Impact | Confidence | Effort | RICE |
|---|---|---|---|---|---|
| Migracao Node.js 20 | 8 devs | 2 | 90% | 4 | 360 |
| Cache Redis | Todos usuarios | 3 | 80% | 6 | 320 |
| Auditoria OWASP | Todos usuarios | 3 | 70% | 3 | 560 |

**Regras de priorizacao:**
- Iniciativas de seguranca critica tem prioridade maxima, independente de RICE
- Divida tecnica que bloqueia features de negocio deve ser quitada primeiro
- Migracoes EOL (end of life) devem ter deadline fixo

### Passo 4 — Estrutura do Roadmap

Crie o roadmap em formato visual e textual:

```markdown
# Roadmap Tecnico — {Nome do Projeto}

## Visao (6-12 meses)
{Descricao da visao tecnica em 2-3 frases}

## Temas e Iniciativas

### Q1 — {Meses}
| Iniciativa | Owner | Status | Sprint Target |
|---|---|---|---|
| Auditoria OWASP e correcoes | @dev1 | 🔴 Nao iniciado | S1-S3 |
| Migracao Node.js 20 | @dev2 | 🟡 Em planejamento | S2-S4 |
| Setup de observabilidade (OpenTelemetry) | @dev3 | 🟢 Em andamento | S1-S2 |

### Q2 — {Meses}
| Iniciativa | Owner | Status | Sprint Target |
|---|---|---|---|
| Cache distribuido Redis | @dev1 | ⚪ Backlog | S5-S7 |
| Design system v1 | @dev2 | ⚪ Backlog | S5-S8 |
| Otimizacao de queries criticas | @dev3 | ⚪ Backlog | S6-S7 |

### Q3 — {Meses}
| Iniciativa | Owner | Status | Sprint Target |
|---|---|---|---|
| Migracao para microservicos (modulo X) | @dev1 | ⚪ Backlog | S9-S12 |
| Criptografia em repouso | @dev2 | ⚪ Backlog | S9-S10 |

### Q4 — {Meses}
| Iniciativa | Owner | Status | Sprint Target |
|---|---|---|---|
| Avaliacao de nova cloud provider | @dev3 | ⚪ Backlog | S13-S14 |
| Refinamento de custos e FinOps | @dev1 | ⚪ Backlog | S13-S14 |

## Metricas de Sucesso

| Metrica | Baseline | Target | Deadline |
|---|---|---|---|
| Latencia p95 API | 450ms | < 200ms | Q2 |
| Coverage de testes | 45% | > 80% | Q3 |
| Vulnerabilidades criticas | 3 | 0 | Q1 |
| Tempo medio de build CI | 12min | < 5min | Q2 |
| MTTR (Mean Time To Recovery) | 45min | < 15min | Q3 |

## Riscos e Mitigacoes

| Risco | Probabilidade | Impacto | Mitigacao |
|---|---|---|---|
| Indisponibilidade durante migracao Node.js | Media | Alto | Canary deploy, rollback automatico, testes de carga |
| Falta de expertise em Redis | Baixa | Medio | Treinamento interno, consultoria pontual |
| Atraso no design system bloqueia frontend | Media | Alto | Definir MVP do design system, aceitar imperfeicoes iniciais |

## Dependencias

- Migracao Node.js 20 deve ser concluida antes de adotar novas features do runtime
- Design system v1 e pre-requisito para refatoracao de telas legadas
- Observabilidade deve estar operacional antes de iniciar otimizacoes de performance
```

### Passo 5 — Rituais de Acompanhamento

Defina a cadencia de revisao:

| Ritual | Frequencia | Participantes | Objetivo |
|---|---|---|---|
| Review de Roadmap | Mensal | Tech Lead, EM, PM | Ajustar prioridades, remover bloqueios |
| Sprint Planning | A cada 2 semanas | Time de engenharia | Quebrar iniciativas em tarefas |
| Tech Radar | Trimestral | Arquitetos, Staff Engineers | Avaliar novas tecnologias |
| Post-Mortem | Apos incidente | Time envolvido | Documentar licoes aprendidas |

### Passo 6 — Comunicacao com Stakeholders

- **Para o time de engenharia:** backlog detalhado, criterios de aceite, definicao de pronto
- **Para Product Managers:** traducao de iniciativas tecnicas em impacto no produto (ex: "cache Redis = reducao de 40% no tempo de carregamento do dashboard")
- **Para C-level:** metricas de sucesso, riscos, investimento necessario

## Output Esperado

- Documento de roadmap tecnico estruturado em markdown
- Priorizacao RICE com justificativas
- Metricas de sucesso com baseline e targets
- Matriz de riscos e mitigacoes
- Cronograma visual (quando possivel, em ferramenta como Miro, Notion ou Jira Roadmaps)
- Comunicacao preparada para diferentes niveis de stakeholders
