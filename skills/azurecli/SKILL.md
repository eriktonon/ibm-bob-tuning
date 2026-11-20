---
name: azurecli
description: "Referência e guia de uso do Azure CLI (az) para. Use quando: provisionar ou gerenciar recursos Azure."
user-invocable: true
---

# Azure CLI — Gerenciamento de Recursos Azure

## Quando Usar

- Provisionar ou gerenciar recursos Azure sem Terraform (scripts rápidos, bootstrap)
- Autenticar e configurar acesso a clusters AKS
- Empurrar imagens para Azure Container Registry (ACR)
- Gerenciar segredos no Key Vault
- Scripts de automação em Bash para pipelines CI/CD

---

## Instalação e Autenticação

```bash
# Instalar Azure CLI
# Linux (apt)
curl -sL https://aka.ms/InstallAzureCLIDeb | sudo bash

# macOS
brew install azure-cli

# Verificar
az version           # >= 2.60 recomendado

# Autenticação interativa (dev local)
az login
az login --tenant <tenant-id>        # tenant específico

# Autenticação por Service Principal (CI/CD)
az login --service-principal \
  --username $ARM_CLIENT_ID \
  --password $ARM_CLIENT_SECRET \
  --tenant $ARM_TENANT_ID

# Listar e selecionar subscriptions
az account list --output table
az account set --subscription "Nome ou ID da subscription"
az account show

# Configurar output padrão
az configure --defaults output=table
az configure --defaults group=meu-resource-group location=brazilsouth
```

---

## Resource Groups

```bash
# Criar
az group create \
  --name rg-pe-agents-ibm-prod \
  --location brazilsouth \
  --tags Environment=production Project=PE-AGENTS-IBM

# Listar
az group list --output table

# Verificar se existe
az group exists --name rg-pe-agents-ibm-prod

# Deletar (⚠️ remove TODOS os recursos dentro)
az group delete --name rg-pe-agents-ibm-prod --yes --no-wait

# Obter ID
az group show --name rg-pe-agents-ibm-prod --query id --output tsv
```

---

## AKS — Azure Kubernetes Service

```bash
# Criar cluster AKS
az aks create \
  --resource-group rg-pe-agents-ibm-prod \
  --name aks-pe-agents-ibm-prod \
  --node-count 3 \
  --node-vm-size Standard_D4s_v3 \
  --min-count 2 \
  --max-count 10 \
  --enable-cluster-autoscaler \
  --kubernetes-version 1.29 \
  --network-plugin azure \
  --load-balancer-sku standard \
  --enable-managed-identity \
  --generate-ssh-keys \
  --tags Environment=production

# Obter kubeconfig
az aks get-credentials \
  --resource-group rg-pe-agents-ibm-prod \
  --name aks-pe-agents-ibm-prod \
  --overwrite-existing

# Listar clusters
az aks list --output table

# Ver versões disponíveis
az aks get-versions --location brazilsouth --output table

# Atualizar versão do cluster
az aks upgrade \
  --resource-group rg-pe-agents-ibm-prod \
  --name aks-pe-agents-ibm-prod \
  --kubernetes-version 1.30

# Escalar node pool manualmente
az aks scale \
  --resource-group rg-pe-agents-ibm-prod \
  --name aks-pe-agents-ibm-prod \
  --node-count 5

# Adicionar node pool
az aks nodepool add \
  --resource-group rg-pe-agents-ibm-prod \
  --cluster-name aks-pe-agents-ibm-prod \
  --name workerpool \
  --node-count 2 \
  --node-vm-size Standard_D8s_v3 \
  --mode User

# Parar / iniciar cluster (economizar custo em dev)
az aks stop  --resource-group rg-pe-agents-ibm-prod --name aks-pe-agents-ibm-dev
az aks start --resource-group rg-pe-agents-ibm-prod --name aks-pe-agents-ibm-dev

# Deletar cluster
az aks delete \
  --resource-group rg-pe-agents-ibm-prod \
  --name aks-pe-agents-ibm-prod \
  --yes --no-wait
```

---

## ACR — Azure Container Registry

```bash
# Criar registry
az acr create \
  --resource-group rg-pe-agents-ibm-prod \
  --name acrpe-agents-ibm \
  --sku Standard \
  --admin-enabled false

# Autenticar Docker no ACR
az acr login --name acrpe-agents-ibm

# Build direto no ACR (sem Docker local)
az acr build \
  --registry acrpe-agents-ibm \
  --image myapp:$GIT_SHA \
  .

# Listar images e tags
az acr repository list --name acrpe-agents-ibm --output table
az acr repository show-tags --name acrpe-agents-ibm --repository myapp --output table

# Integrar ACR ao AKS (permissão de pull)
az aks update \
  --resource-group rg-pe-agents-ibm-prod \
  --name aks-pe-agents-ibm-prod \
  --attach-acr acrpe-agents-ibm

# Purgar images antigas (manter apenas últimas 10 tags)
az acr run \
  --registry acrpe-agents-ibm \
  --cmd "acr purge --filter 'myapp:.*' --keep 10 --ago 7d" \
  /dev/null
```

---

## Key Vault

```bash
# Criar Key Vault
az keyvault create \
  --resource-group rg-pe-agents-ibm-prod \
  --name kv-pe-agents-ibm-prod \
  --location brazilsouth \
  --enable-rbac-authorization true

# Dar acesso a um Service Principal
az role assignment create \
  --role "Key Vault Secrets Officer" \
  --assignee $APP_OBJECT_ID \
  --scope $(az keyvault show --name kv-pe-agents-ibm-prod --query id --output tsv)

# Secrets
az keyvault secret set \
  --vault-name kv-pe-agents-ibm-prod \
  --name DATABASE-PASSWORD \
  --value "senha_secreta"

az keyvault secret show \
  --vault-name kv-pe-agents-ibm-prod \
  --name DATABASE-PASSWORD \
  --query value \
  --output tsv

az keyvault secret list \
  --vault-name kv-pe-agents-ibm-prod \
  --output table

az keyvault secret delete \
  --vault-name kv-pe-agents-ibm-prod \
  --name DATABASE-PASSWORD

# Keys (para criptografia)
az keyvault key create \
  --vault-name kv-pe-agents-ibm-prod \
  --name mykey \
  --kty RSA \
  --size 2048
```

---

## PostgreSQL Flexível (Managed)

```bash
# Criar servidor PostgreSQL
az postgres flexible-server create \
  --resource-group rg-pe-agents-ibm-prod \
  --name psql-pe-agents-ibm-prod \
  --location brazilsouth \
  --admin-user pgadmin \
  --admin-password "$DB_ADMIN_PASSWORD" \
  --sku-name Standard_D4s_v3 \
  --tier GeneralPurpose \
  --storage-size 128 \
  --version 16 \
  --high-availability Enabled

# Criar banco de dados
az postgres flexible-server db create \
  --resource-group rg-pe-agents-ibm-prod \
  --server-name psql-pe-agents-ibm-prod \
  --database-name pe-agents-ibm

# Regra de firewall (permitir AKS)
az postgres flexible-server firewall-rule create \
  --resource-group rg-pe-agents-ibm-prod \
  --name psql-pe-agents-ibm-prod \
  --rule-name AllowAKS \
  --start-ip-address 10.0.1.0 \
  --end-ip-address 10.0.1.255

# Conectar via CLI
az postgres flexible-server connect \
  --name psql-pe-agents-ibm-prod \
  --admin-user pgadmin \
  --database-name pe-agents-ibm
```

---

## Networking

```bash
# Criar VNet e subnet
az network vnet create \
  --resource-group rg-pe-agents-ibm-prod \
  --name vnet-pe-agents-ibm-prod \
  --address-prefix 10.0.0.0/16

az network vnet subnet create \
  --resource-group rg-pe-agents-ibm-prod \
  --vnet-name vnet-pe-agents-ibm-prod \
  --name subnet-aks \
  --address-prefix 10.0.1.0/24

az network vnet subnet create \
  --resource-group rg-pe-agents-ibm-prod \
  --vnet-name vnet-pe-agents-ibm-prod \
  --name subnet-db \
  --address-prefix 10.0.2.0/24

# Network Security Group
az network nsg create \
  --resource-group rg-pe-agents-ibm-prod \
  --name nsg-aks

az network nsg rule create \
  --resource-group rg-pe-agents-ibm-prod \
  --nsg-name nsg-aks \
  --name AllowHTTPS \
  --priority 100 \
  --protocol Tcp \
  --destination-port-ranges 443 \
  --access Allow
```

---

## Service Principal para CI/CD

```bash
# Criar SP com escopo de resource group
az ad sp create-for-rbac \
  --name sp-pe-agents-ibm-cicd \
  --role Contributor \
  --scopes /subscriptions/$SUBSCRIPTION_ID/resourceGroups/rg-pe-agents-ibm-prod \
  --output json

# Saída — armazenar como secret no GitHub/Azure DevOps:
# {
#   "appId": "...",        → AZURE_CLIENT_ID
#   "displayName": "...",
#   "password": "...",     → AZURE_CLIENT_SECRET
#   "tenant": "..."        → AZURE_TENANT_ID
# }

# Listar SPs
az ad sp list --display-name sp-pe-agents-ibm-cicd --output table

# Resetar credenciais
az ad sp credential reset --id $APP_ID
```

---

## Consultas Úteis com JMESPath

```bash
# Listar apenas resource groups na region brazilsouth
az group list --query "[?location=='brazilsouth'].name" --output tsv

# IP público de um recurso
az network public-ip show \
  --resource-group rg-pe-agents-ibm-prod \
  --name pip-lb \
  --query ipAddress \
  --output tsv

# Todos os pods em running no AKS (via kubectl após get-credentials)
kubectl get pods --all-namespaces --field-selector=status.phase=Running

# Custo estimado de um resource group
az consumption usage list \
  --billing-period-name $(date +%Y%m) \
  --query "[?resourceGroup=='rg-pe-agents-ibm-prod']" \
  --output table
```

---

## Output Esperado

1. Comandos organizados por recurso para o ambiente alvo
2. Service Principal criado e configurado para CI/CD
3. AKS com kubeconfig atualizado localmente
4. ACR integrado ao AKS com permissão de pull
5. Key Vault com RBAC configurado para a aplicação
