---
name: k8s-policy-check
description: "Valida manifests Kubernetes contra policies de. Use quando: validar manifests antes de aplicar em."
user-invocable: true
---

# Policy-as-code com Polaris

## Quando Usar

- Após gerar manifests com a skill `scaffold-k8s`
- Antes de aplicar manifests em produção
- Ao auditar manifests existentes de um cluster legado
- Como gate de qualidade em pipeline CI/CD

## Pré-requisitos

```bash
# Verificar se polaris está instalado
polaris --version || echo "Instale: https://polaris.docs.fairwinds.com/installation/"
```

> **Nota:** Polaris (Apache-2.0) é uma policy engine de código aberto que audita manifests Kubernetes contra 30+ checks de segurança, resiliência e boas práticas. Pode rodar como CLI, dashboard ou admission controller webhook.

## Procedimento

### Passo 1 — Auditoria local de manifests

```bash
# Auditar todos os manifests da pasta k8s/
polaris audit --audit-path k8s/ --format=score

# Output em formato JSON para parse automatizado
polaris audit --audit-path k8s/ --format=json > polaris-report.json

# Auditar kustomization build (incluindo overlays)
kustomize build k8s/overlays/prod/ | polaris audit --audit-path - --format=score
```

### Passo 2 — Interpretar o score

Polaris retorna um score de 0-100. Targets recomendados:

| Score | Classificação | Ação |
|---|---|---|
| 90-100 | Excelente | Pronto para produção |
| 75-89 | Bom | Corrigir warnings antes de produção |
| 50-74 | Regular | Corrigir errors antes de staging |
| 0-49 | Crítico | Bloqueia deploy |

### Passo 3 — Analisar findings por categoria

Polaris valida 5 categorias:

#### Categoria 1 — Security
- `hostNetwork` não deve ser `true`
- `hostPID` não deve ser `true`
- `hostIPC` não deve ser `true`
- `runAsNonRoot` deve ser `true`
- `readOnlyRootFilesystem` deve ser `true`
- `allowPrivilegeEscalation` deve ser `false`
- `capabilities.drop` deve incluir `ALL`

#### Categoria 2 — Networking
- NetworkPolicy deve estar definida
- `hostPort` não deve ser usado

#### Categoria 3 — Resources
- `resources.requests.cpu` deve estar definido
- `resources.requests.memory` deve estar definido
- `resources.limits.cpu` deve estar definido
- `resources.limits.memory` deve estar definido

#### Categoria 4 — Reliability
- `livenessProbe` deve estar configurado
- `readinessProbe` deve estar configurado
- `replicas` deve ser ≥ 2 em produção
- `imagePullPolicy` não deve ser `Always` (usar `IfNotPresent`)

#### Categoria 5 — Images
- Image tag não deve ser `latest`
- Image deve ter versão pinada

### Passo 4 — Custom policies (avançado)

Polaris suporta policies customizadas via JSON Schema. Crie um arquivo de config:

```yaml
# polaris-config.yaml
checks:
  resources:
    minimumReplicas: 3  # exijir 3 réplicas mínimo em produção
  security:
    runAsNonRoot: true
  reliability:
    multipleReplicas: true
```

```bash
# Aplicar com config customizada
polaris audit --audit-path k8s/ --config polaris-config.yaml --format=score
```

### Passo 5 — Integração com CI/CD

```yaml
# Exemplo de step em pipeline (GitHub Actions)
- name: Polaris audit
  run: |
    kustomize build k8s/overlays/prod/ | polaris audit --audit-path - --format=score
    SCORE=$(kustomize build k8s/overlays/prod/ | polaris audit --audit-path - --format=json | jq .score)
    if [ "$SCORE" -lt 80 ]; then
      echo "Polaris score $SCORE abaixo do mínimo (80)"
      exit 1
    fi
```

## Remediação comum

```yaml
# ❌ Violando múltiplas policies
spec:
  replicas: 1
  template:
    spec:
      containers:
        - name: app
          image: myapp:latest
          # sem resources, sem probes, sem securityContext

# ✅ Conformidade com policies
spec:
  replicas: 3  # reliability: multipleReplicas
  template:
    spec:
      securityContext:           # security: runAsNonRoot
        runAsNonRoot: true
        runAsUser: 1000
      containers:
        - name: app
          image: myapp:1.0.0     # images: tag pinada
          resources:             # resources: requests e limits
            requests:
              cpu: "100m"
              memory: "128Mi"
            limits:
              cpu: "500m"
              memory: "512Mi"
          securityContext:
            readOnlyRootFilesystem: true
            allowPrivilegeEscalation: false
            capabilities:
              drop: ["ALL"]
          livenessProbe:         # reliability: livenessProbe
            httpGet:
              path: /health/live
              port: 8080
          readinessProbe:       # reliability: readinessProbe
            httpGet:
              path: /health/ready
              port: 8080
```

## Output Esperado

- Score do polaris (0-100)
- Lista de findings por severidade (error, warning, info)
- Categoria de cada finding (security, networking, resources, reliability, images)
- Manifests corrigidos para resolver findings de nível error
- Pendências documentadas para findings de nível warning
- Config de custom policies (quando aplicável)
