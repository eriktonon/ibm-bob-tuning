---
name: build-to-manage
description: "Modelo Build to Manage (IBM SRE/ITOps): sistemas nascem. Use quando: desenhar servicos para producao."
user-invocable: true
---

# Build to Manage

## O Que E

`Build to Manage` e um modelo de engenharia operacional: a aplicacao nao deve apenas funcionar; ela deve nascer **gerenciavel** em producao.

Neste contexto, o objetivo e garantir que qualquer servico entregue pelo time possua, desde o inicio:

- observabilidade suficiente para diagnostico rapido
- automacao para reduzir toil operacional
- runbooks e ownership claros
- metas de confiabilidade explicitas
- mecanismos de rollback e recuperacao
- capacidade minima de operar sob falha

No ecossistema IBM, isso se alinha ao pensamento de **SRE**, **observability**, **ITOps/AIOps** e **platform engineering**, mesmo quando o nome formal do framework nao aparece centralizado em uma unica pagina publica.

---

## Quando Usar

Use esta skill quando:

- uma aplicacao nova estiver sendo preparada para producao
- uma modernizacao precisar incluir readiness operacional
- o agente SRE for gerar Dockerfile, Kubernetes e pipeline, mas ainda faltar operabilidade real
- o time precisar definir o que um servico deve entregar para ser suportavel em runtime
- houver incidentes recorrentes e o problema for falta de telemetria, automacao, runbook ou ownership

Nao use esta skill para substituir troubleshooting pontual. Para incidentes em andamento, use `k8s-troubleshoot` ou o agente SRE Investigador.

---

## Princípios

1. **Operabilidade e requisito de design**
   O sistema deve ser projetado para ser operado, nao apenas para ser desenvolvido.

2. **Observabilidade antes da crise**
   Logs, metricas e traces precisam existir antes do primeiro incidente.

3. **Automacao acima de toil**
   Toda acao operacional repetitiva deve virar script, job, pipeline ou runbook automatizavel.

4. **Falha e parte do modelo**
   O servico precisa prever degradacao, retry, timeout, rollback e recuperacao.

5. **Ownership explicito**
   Cada servico deve ter dono tecnico, superficie de responsabilidade e caminho de escalacao.

6. **Feedback operacional volta para engenharia**
   Incidentes, alertas ruins e gargalos viram backlog tecnico.

---

## Procedimento

### Passo 1 — Mapear o Servico Operavel

Identifique:

- nome do servico
- objetivo de negocio
- dependencias externas e internas
- entradas e saidas principais
- dono tecnico
- ambiente alvo

Produza um inventario curto:

```markdown
## Inventario Operacional
- Servico:
- Objetivo:
- Dono tecnico:
- Dependencias criticas:
- Ambientes:
- Impacto de indisponibilidade:
```

### Passo 2 — Definir Sinais Operacionais

Para cada fluxo critico, defina:

- **SLIs**: disponibilidade, latencia, taxa de erro, throughput
- **SLOs**: metas e limiares
- sinais de saturacao: CPU, memoria, fila, conexoes, pool, backlog
- eventos de negocio relevantes para observacao

Exemplo:

```markdown
## SLOs Minimos
- Disponibilidade: 99.9%
- p95 de latencia: < 300ms
- Taxa de erro 5xx: < 1%
- Health check: /health e /ready obrigatorios
```

### Passo 3 — Especificar Telemetria

Defina o minimo obrigatorio:

- logs estruturados com correlation/request ID
- metricas de servico e dependencia
- traces distribuidos para fluxos criticos
- dashboards para disponibilidade, latencia, saturacao e erro budget burn
- alertas acionaveis, com dono e severidade

Checklist:

- [ ] logs estruturados
- [ ] metricas expostas
- [ ] trace IDs propagados
- [ ] dashboard inicial definido
- [ ] alertas com thresholds e owner

### Passo 4 — Garantir Automacao Operacional

Defina como o servico sera operado sem toil manual excessivo:

- deploy automatizado
- rollback automatizado ou semi-automatizado
- restart seguro
- scale up/down controlado
- backup e restore quando aplicavel
- smoke test pos-deploy

Checklist:

- [ ] pipeline com deploy reproduzivel
- [ ] rollback documentado
- [ ] smoke test pos-deploy
- [ ] processo de restauracao conhecido

### Passo 5 — Desenhar para Falha

Verifique se o servico tem:

- liveness e readiness probes
- timeout e retry com criterio
- circuit breaker ou bulkhead quando fizer sentido
- rate limit ou protecao contra saturacao
- degradacao aceitavel quando dependencia cai
- comportamento explicito em caso de indisponibilidade de banco, fila ou API externa

### Passo 6 — Criar Runbooks

Para os top incidentes esperados, produza runbooks curtos:

- aplicacao nao sobe
- readiness falhando
- latencia alta
- erro 5xx acima do normal
- dependencia externa indisponivel
- deploy ruim / rollback
- saturacao de CPU ou memoria

Formato minimo:

```markdown
## Runbook: {incidente}

### Sintoma
### Como confirmar
### Causa provavel
### Acao imediata
### Validacao pos-correcao
### Escalacao
```

### Passo 7 — Planejar Capacidade e Recuperacao

Defina:

- baseline de carga esperada
- estrategia de escalabilidade
- limites operacionais conhecidos
- RTO e RPO quando aplicavel
- estrategia de backup, restore e failover

### Passo 8 — Fechar o Gate Build to Manage

Antes de considerar o trabalho pronto para producao, confirme:

- [ ] inventario operacional pronto
- [ ] SLOs definidos
- [ ] telemetria minima definida
- [ ] alertas acionaveis definidos
- [ ] automacao operacional prevista
- [ ] runbooks iniciais escritos
- [ ] estrategia de rollback conhecida
- [ ] capacidade e recuperacao consideradas

Se mais de 3 itens criticos estiverem ausentes, a entrega nao esta pronta para producao.

---

## Output Esperado

- checklist Build to Manage preenchido
- backlog de gaps operacionais
- runbooks minimos
- definicao de SLOs e telemetria obrigatoria
- plano de automacao operacional e rollback
