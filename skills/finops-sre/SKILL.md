---
name: finops-sre
description: "Praticas de FinOps alinhadas a SRE: visibilidade de. Use quando: auditar gastos cloud."
user-invocable: true
---

# FinOps SRE — Eficiência Financeira em Nuvem

## O Que É

FinOps é a prática de trazer responsabilidade financeira para a nuvem, permitindo que equipes de engenharia (especialmente SRE e Platform Engineering) tomem decisões informadas sobre custo-benefício sem sacrificar confiabilidade ou performance.

Esta skill integra FinOps com SRE no ecossistema IBM, garantindo que todo provisionamento de infraestrutura nasça com:

- **visibilidade de custo** desde o primeiro deploy
- **otimização contínua** de recursos subutilizados
- **budgets e alertas** que previnem surpresas na fatura
- **cost allocation tags** que permitem atribuir gastos a times/projetos
- **estratégia de pricing** (reserved, savings plans, spot) alinhada aos SLOs
- **cultura de ownership financeiro** compartilhada entre engenharia e finanças

---

## Quando Usar

Use esta skill quando:

- provisionar nova infraestrutura em Azure, AWS ou GCP e precisar estimar custos
- auditar gastos de nuvem e identificar oportunidades de economia
- otimizar custos de clusters Kubernetes (AKS/EKS/GKE) sem degradar SLOs
- definir budgets, alertas de custo e políticas de governança financeira
- criar dashboards de FinOps para visibilidade de gastos por time/projeto
- planejar estratégia de reserved instances, savings plans ou spot instances
- revisar Dockerfiles e manifests Kubernetes do ponto de vista de custo
- implementar auto-scaling com políticas baseadas em custo
- realizar rightsizing de VMs, bancos de dados e serviços gerenciados
- preparar relatórios de eficiência financeira para stakeholders

Não use esta skill para:
- troubleshooting de incidentes em andamento (use `k8s-troubleshoot`)
- provisionamento puro de IaC sem foco em custo (use `terraform`)
- revisão de segurança (use `revisao-owasp`)

---

## Princípios

1. **Custo é uma métrica de SRE**
   Assim como latência e disponibilidade, custo deve ser observado, alertado e otimizado continuamente.

2. **Visibilidade antes da otimização**
   Não é possível otimizar o que não é medido. Tags, budgets e dashboards vêm antes de cortes.

3. **Rightsizing > Cutting**
   Reduzir recursos subutilizados é mais seguro que cortar serviços. Sempre basear em dados reais de consumo.

4. **Pricing estratégico alinhado a SLOs**
   Reserved instances para baseline estável, spot para workloads tolerantes a interrupção, on-demand para picos.

5. **Cultura de ownership compartilhado**
   Engenharia decide a arquitetura, FinOps fornece dados e guardrails, ambos respondem pelo custo.

6. **Eficiência sem degradação**
   Toda otimização de custo deve ser validada contra SLOs. Reduzir custo à custa de confiabilidade é anti-padrão.

7. **Automação de governança**
   Policies, budget alerts e auto-scaling devem ser automatizados, não depender de revisão manual.

---

## Procedimento

### Passo 1 — Inventário de Custos

Identifique e documente:

```markdown
## Inventário FinOps
- Cloud Provider: (Azure / AWS / GCP)
- Assinatura/Conta/Project: 
- Serviços em uso: (AKS, RDS, S3, Cloud Run, etc.)
- Gasto mensal atual: R$ ___
- Distribuição por serviço:
  - Compute: ___%
  - Storage: ___%
  - Network: ___%
  - Managed services: ___%
- Tags existentes: (sim/não/parcial)
- Budgets configurados: (sim/não)
- Reserved instances: (sim/não/quais)
```

### Passo 2 — Tagging e Cost Allocation

Garanta que todos os recursos tenham tags consistentes para atribuição de custo:

#### Tags mínimas recomendadas

```yaml
# Estrutura de tags obrigatórias
tags:
  environment: "production"      # dev | staging | production
  team: "platform-engineering"   # time responsável
  project: "pe-agents-ibm"       # projeto/produto
  cost-center: "CC-12345"        # centro de custo
  owner: "tech-lead@ibm.com"     # dono técnico
  criticality: "high"            # low | medium | high | critical
  created-by: "terraform"        # terraform | manual | pipeline
```

#### Aplicação por provider

**Azure (az CLI):**
```bash
# Listar recursos sem tags obrigatórias
az resource list --query "[?tags.environment == null].{name:name, rg:resourceGroup}" -o table

# Aplicar tags em massa
az tag create --resource-id <resource-id> --tags environment=production team=platform

# Policy para exigir tags
az policy assignment create --name "require-env-tag" \
  --policy "required-tags" \
  --params '{"tagName":{"value":"environment"}}'
```

**AWS (aws CLI):**
```bash
# Listar recursos sem tags
aws resourcegroupstaggingapi get-resources --tag-filters Key=environment,Values=production

# Aplicar tags
aws ec2 create-tags --resources i-123456 --tags Key=environment,Value=production

# Cost Explorer por tag
aws ce get-cost-and-usage --time-period Start=2026-01-01,End=2026-06-01 \
  --granularity MONTHLY --group-by Type=TAG,Key=team
```

**GCP (gcloud):**
```bash
# Listar recursos sem labels
gcloud compute instances list --filter="labels.environment:null"

# Aplicar labels
gcloud compute instances update <instance> --update-labels environment=production,team=platform

# Exportar fatura para BigQuery
gcloud billing export describe --billing-account=<account-id>
```

### Passo 3 — Análise de Gastos e Oportunidades

#### Dashboard de FinOps

Crie um dashboard com as seguintes visualizações:

1. **Gasto total mensal** (trend dos últimos 6 meses)
2. **Gasto por serviço** (top 10)
3. **Gasto por team/project** (via tags)
4. **Gasto por ambiente** (dev vs staging vs prod)
5. **Custo por unidade de negócio** (cost per request, cost per user, cost per transaction)
6. **Anomalias de custo** (spikes > 20% vs média)
7. **Recursos idle/orphaned** (sem tráfego há 30+ dias)

#### Queries de análise

**Azure Cost Management:**
```bash
# Gasto por resource group nos últimos 30 dias
az costmanagement query --type ActualCost \
  --timeframe MonthToDate \
  --dimension ResourceGroup \
  --scope "/subscriptions/<sub-id>"

# Gasto por tag team
az costmanagement query --type ActualCost \
  --timeframe Custom --start-date 2026-05-01 --end-date 2026-06-01 \
  --dimension TagKey --tag-key team \
  --scope "/subscriptions/<sub-id>"
```

**AWS Cost Explorer:**
```bash
# Gasto por serviço
aws ce get-cost-and-usage \
  --time-period Start=2026-05-01,End=2026-06-01 \
  --granularity MONTHLY \
  --metrics BlendedCost \
  --group-by Type=DIMENSION,Key=SERVICE

# Gasto por tag team
aws ce get-cost-and-usage \
  --time-period Start=2026-05-01,End=2026-06-01 \
  --granularity MONTHLY \
  --metrics BlendedCost \
  --group-by Type=TAG,Key=team
```

**GCP Billing Export (BigQuery):**
```sql
SELECT
  service.description AS service,
  SUM(cost) AS total_cost
FROM `project.dataset.gcp_billing_export_v1_XXXXXX`
WHERE invoice.month = '202606'
GROUP BY service
ORDER BY total_cost DESC
LIMIT 10;
```

### Passo 4 — Rightsizing

#### Kubernetes (AKS/EKS/GKE)

```bash
# Verificar utilização real de recursos
kubectl top nodes
kubectl top pods --all-namespaces --sort-by=cpu

# Identificar pods com requests excessivos vs uso real
kubectl get pods -A -o json | jq '.items[] | {
  name: .metadata.name,
  ns: .metadata.namespace,
  cpu_request: .spec.containers[].resources.requests.cpu,
  cpu_usage: "verificar com kubectl top",
  mem_request: .spec.containers[].resources.requests.memory
}'

# Recomendar rightsizing
# Se request CPU > 2x uso real médio (7 dias) → reduzir request
# Se request memory > 1.5x uso real pico → reduzir request
# Se limite CPU nunca atingido → remover limite ou reduzir
```

#### VMs / Managed Services

```bash
# Azure — recomendar SKU menor
az vm list-usage --location eastus -o table
az vm list-skus --location eastus --resource-type virtualMachines -o table

# Verificar métricas de uso
az monitor metrics list --resource <vm-id> \
  --metric "Percentage CPU" --interval PT1H \
  --time-grain PT1H --start 2026-05-01 --end 2026-06-01

# AWS — recomendar instância menor
aws ec2 describe-instances --query 'Reservations[].Instances[].{Id:InstanceId,Type:InstanceType,State:State.Name}' -o table

# Verificar CloudWatch metrics
aws cloudwatch get-metric-statistics \
  --namespace AWS/EC2 --metric-name CPUUtilization \
  --dimensions Name=InstanceId,Value=i-123456 \
  --start-time 2026-05-01 --end-time 2026-06-01 \
  --period 3600 --statistics Average
```

#### Tabela de rightsizing

```markdown
| Recurso | SKU Atual | Uso Médio (7d) | SKU Recomendado | Economia/mês | Risco SLO |
|---|---|---|---|---|---|
| VM-app-01 | Standard_D4s_v3 | CPU 12%, RAM 35% | Standard_D2s_v3 | R$ 280 | Baixo |
| AKS node-1 | Standard_DS2_v2 | CPU 40%, RAM 60% | Manter | — | — |
| RDS-prod | db.r5.xlarge | CPU 8%, Conexões 15 | db.r5.large | R$ 450 | Baixo |
| S3 bucket | Standard | Acesso < 1/mês | Glacier | R$ 30 | Nenhum |
```

### Passo 5 — Estratégia de Pricing

#### Reserved Instances / Savings Plans

```markdown
## Matriz de Pricing

| Workload | Padrão de uso | Pricing recomendado | Cobertura | Economia estimada |
|---|---|---|---|---|
| AKS nodes (baseline) | 24/7 estável | Reserved 1yr | 70% do baseline | ~40% |
| Batch processing | Horário comercial | Reserved 1yr | 100% | ~40% |
| CI/CD runners | Esporádico, tolerante a interrupção | Spot | 100% | ~70% |
| Picos de tráfego | Variável | On-demand | 30% | 0% |
| Banco de dados prod | 24/7 crítico | Reserved 3yr | 100% | ~60% |
```

#### Spot Instances para workloads tolerantes

```yaml
# Kubernetes — node pool spot para batch/CI
apiVersion: apps/v1
kind: Deployment
metadata:
  name: ci-runner
  labels:
    finops.ibm.com/pricing: spot
    finops.ibm.com/interruptible: "true"
spec:
  replicas: 4
  template:
    spec:
      nodeSelector:
        kubernetes.azure.com/scalesetpriority: spot
      tolerations:
        - key: kubernetes.azure.com/scalesetpriority
          operator: Equal
          value: spot
          effect: NoSchedule
      containers:
        - name: runner
          resources:
            requests:
              cpu: 500m
              memory: 1Gi
            limits:
              cpu: 1000m
              memory: 2Gi
```

### Passo 6 — Budgets e Alertas

#### Azure Budget Alerts

```bash
# Criar budget mensal
az consumption budget create --amount 5000 \
  --budget-name "monthly-budget" \
  --category Cost \
  --time-grain Monthly \
  --start-date 2026-06-01 \
  --time-period start=2026-06-01 end=2026-12-31

# Alerta em 80%
az consumption budget create --amount 5000 \
  --budget-name "monthly-budget" \
  --notifications '[
    {"contactEmails":["sre@ibm.com"],"contactRoles":["Owner"],"threshold":80,"operator":"GreaterThan"}
  ]'
```

#### AWS Budget Alerts

```bash
# Criar budget
aws budgets create-budget \
  --account-id 123456789012 \
  --budget '{"BudgetName":"monthly-budget","BudgetLimit":{"Amount":"5000","Unit":"USD"},"BudgetType":"COST","TimeUnit":"MONTHLY"}' \
  --notifications-with-subscribers '[{"Notification":{"NotificationType":"ACTUAL","ComparisonOperator":"GREATER_THAN","Threshold":80},"Subscribers":[{"SubscriptionType":"EMAIL","Address":"sre@ibm.com"}]}]'
```

#### GCP Budget Alerts

```bash
# Criar budget via gcloud
gcloud billing budgets create \
  --billing-account=<account-id> \
  --display-name="monthly-budget" \
  --budget-amount=5000USD \
  --threshold-rule=percent=80 \
  --threshold-rule=percent=100 \
  --notification-channel=<channel-id>
```

### Passo 7 — Otimização de Kubernetes

#### HPA com políticas de custo

```yaml
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: app-hpa
  labels:
    finops.ibm.com/optimized: "true"
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: app
  minReplicas: 2          # mínimo para SLO
  maxReplicas: 10         # teto de custo
  metrics:
    - type: Resource
      resource:
        name: cpu
        target:
          type: Utilization
          averageUtilization: 70   # escalar só quando realmente necessário
  behavior:
    scaleDown:
      stabilizationWindowSeconds: 300  # não reduzir imediatamente
      policies:
        - type: Percent
          value: 25                    # reduzir no máximo 25% por vez
          periodSeconds: 60
```

#### Cluster Autoscaler com balanceamento de custo

```markdown
## Configuração recomendada

- Node pool on-demand: min=2, max=4 (baseline estável)
- Node pool spot: min=0, max=6 (expansão para picos)
- Prioridade: spot > on-demand (se tolerante a interrupção)
- Taint/toleration: workloads críticos só em on-demand
```

#### Limpeza de recursos idle

```bash
# PVCs não utilizados
kubectl get pvc --all-namespaces | grep Pending

# Services sem endpoints
kubectl get svc --all-namespaces -o json | jq '.items[] | select(.spec.type=="LoadBalancer") | select(.status.loadBalancer.ingress == null) | .metadata.name'

# Pods em CrashLoopBackOff consumindo recursos
kubectl get pods -A --field-selector=status.phase=Pending

# Images não utilizadas em nodes
crictl rmi --prune
```

### Passo 8 — Relatório de FinOps

Gere um relatório periódico com:

```markdown
# Relatório FinOps — <Mês/Ano>

## Resumo Executivo
- Gasto total: R$ ___
- vs mês anterior: ___% (↑/↓)
- vs budget: ___% (dentro/acima/abaixo)
- Economia realizada: R$ ___ (rightsizing + reserved)

## Gasto por Time
| Team | Gasto | % do total | Trend | Status |
|---|---|---|---|---|
| Platform Eng | R$ ___ | ___% | ↑ | ⚠️ Acima do budget |
| Backend | R$ ___ | ___% | ↓ | ✅ Dentro |
| Frontend | R$ ___ | ___% | → | ✅ Dentro |

## Top 5 Oportunidades de Economia
1. <recurso> — R$ ___/mês (rightsizing)
2. <recurso> — R$ ___/mês (reserved instance)
3. <recurso> — R$ ___/mês (idle, deletar)
4. <recurso> — R$ ___/mês (migrar para spot)
5. <recurso> — R$ ___/mês (storage tier)

## Ações Realizadas
- [x] Rightsizing de 3 VMs → R$ ___/mês economizado
- [x] Reserved instance para AKS baseline → R$ ___/mês
- [x] Limpeza de 12 recursos idle → R$ ___/mês
- [ ] Migrar CI/CD para spot → R$ ___/mês projetado

## SLOs vs Custo
| Serviço | SLO | Custo atual | Custo por request | Status |
|---|---|---|---|---|
| API | 99.9% | R$ ___ | R$ 0.00___ | ✅ |
| Frontend | 99.95% | R$ ___ | R$ 0.00___ | ✅ |

## Próximos Passos
- [ ] Implementar budget alert em 90%
- [ ] Revisar reserved instances (renovação em 60 dias)
- [ ] Avaliar Savings Plans vs Reserved
```

---

## Ferramentas e Integrações

| Ferramenta | Função | Provider |
|---|---|---|
| Azure Cost Management | Dashboard e budgets | Azure |
| AWS Cost Explorer | Análise de gastos | AWS |
| AWS Budgets | Alertas de budget | AWS |
| GCP Billing Export | Dados de fatura para BigQuery | GCP |
| Kubecost | Custo por namespace/workload | Multi-cloud K8s |
| OpenCost | Open-source cost monitoring | K8s |
| Cloud Custodian | Policies de governança | Multi-cloud |
| Infracost | Estimativa de custo em Terraform | Multi-cloud |
| Terraform Cost Estimation | Previsão antes do apply | Multi-cloud |

### Infracost — Estimativa em IaC

```bash
# Instalar
brew install infracost

# Analisar Terraform
infracost breakdown --path infra/environments/production/ \
  --usage-file infracost-usage.yml

# Output com custo estimado por recurso
infracost breakdown --path . --format json --out-file cost.json
```

### Kubecost — Custo de Kubernetes

```bash
# Instalar via Helm
helm install kubecost cost-analyzer \
  --repo https://kubecost.github.io/cost-analyzer/ \
  --namespace kubecost --create-namespace \
  --set kubecostToken="..."

# Acessar dashboard
kubectl port-forward -n kubecost svc/kubecost-frontend 9090:80
# Abrir http://localhost:9090
```

---

## Checklist de FinOps para Deploy

Antes de qualquer deploy em produção, verifique:

- [ ] Recursos têm tags obrigatórias (environment, team, project, cost-center)
- [ ] SKU/instância escolhida é o menor que atende aos SLOs
- [ ] Requests/limits de Kubernetes baseados em dados reais de consumo
- [ ] HPA configurado com min/max baseado em carga e custo
- [ ] Budget alert configurado para o resource group/projeto
- [ ] Storage tier apropriado (Hot/Cool/Archive/Glacier)
- [ ] Recursos temporários (CI/CD, preview envs) usam spot quando possível
- [ ] Reserved/Savings Plans avaliados para baseline estável
- [ ] Auto-shutdown configurado para ambientes dev/staging
- [ ] Custo estimado documentado no PR/ADR

---

## Integração com Skills PE-AGENTS-IBM

| Skill | Integração |
|---|---|
| `terraform` | Adicionar `infracost` ao pipeline para estimativa antes do apply |
| `scaffold-k8s` | Incluir labels FinOps e HPA com políticas de custo |
| `pipeline-cicd` | Adicionar step de cost estimation no pipeline |
| `build-to-manage` | Incluir custo como métrica observável alongside SLOs |
| `dockerfile-review` | Avaliar tamanho de imagem como fator de custo (registry + pull) |
| `awscli` / `azurecli` / `gcpsdk` | Comandos de cost management integrados |
| `githubactions` | Budget check antes de deploy em produção |
