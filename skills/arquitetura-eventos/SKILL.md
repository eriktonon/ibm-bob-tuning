---
name: arquitetura-eventos
description: "Padroes de arquitetura orientada a eventos e Clean. Use quando: projetar sistemas distribuidos."
user-invocable: true
---

# Skill Arquitetura Orientada a Eventos & Padrões Distribuidos

Esta skill condensa os playbooks arquiteturais validados empiricamente com suites reais de testes no sandbox e persistidos na base vetorial do project context bank e no `.pe-agents-ibm/context.md`:

---

## 1. Decider Pattern Funcional Puro (Event Sourcing & CQRS)

Evita o acoplamento do agregado com ORMs ou frameworks de infraestrutura:
- **`Decide(Command, State) -> Vec<Event>`**: Valida invariantes e regras de negócio de forma determinística, emitindo eventos imutáveis sem efeito colateral.
- **`Evolve(State, Event) -> State`**: Função pura de transição de estado.
- **Reconstituição via Fold**: `events.fold(initial_state, evolve)` reconstrói o estado instantaneamente a partir do event stream.
- **Isolamento Total**: Camada de domínio permanece 100% agnóstica a bancos de dados, storage ou brokers de mensageria.

---

## 2. Transactional Outbox Pattern

Elimina o problema de *dual-write* ao persistir dados e publicar em brokers (Kafka / RabbitMQ / SQS):
1. **Mesma Transação ACID**: A entidade de negócio e o registro em `OutboxMessages` são gravados estritamente na mesma transação de banco.
2. **Consumo Concorrente Sem Contenção**: O worker em background utiliza `FOR UPDATE SKIP LOCKED` (PostgreSQL) ou `WITH (UPDLOCK, ROWLOCK, READPAST)` (.NET / SQL Server) para ler lotes pendentes em paralelo sem lock contention.
3. **Resiliência e Dead Letter Queue**: Falhas repetidas de publicação incrementam `retry_count`; ao atingir o limite (`max_retries`), a mensagem é movida atomicamente para uma tabela de Dead Letter para não envenenar a fila.

---

## 3. Event-Driven Architecture (EDA) & Separação de Eventos

- **Domain Events**: Eventos internos ao agregado / bounded context, não expostos diretamente a outros serviços.
- **Integration Events**: Contratos públicos versionados (ex: `OrderCreatedV1`) enviados ao broker para comunicação entre microsserviços.
- **Idempotência Obrigatória**: Todo consumidor de mensageria deve manter tabela ou cache de deduplicação por `message_id` para suportar semântica *at-least-once*.
- **Consumers como Adaptadores**: Na Clean Architecture, o consumer de mensagens atua na camada externa (Círculo 4), convertendo o payload em um Command de aplicação e despachando via Use Case / Mediator.

---

## 4. API Gateway Cloud-Native & Sliding Window Rate Limiting

- **Defesa de Borda**: Pipeline com autenticação e estrangulamento de requisições executado antes do proxy upstream.
- **Algoritmo Sliding Window**: Calcula `(previous_window_count * weight) + current_window_count`, prevenindo a vulnerabilidade de pico duplo (*burst attack*) presente em janelas fixas tradicionais.
- **Núcleo Agnóstico**: Lógica de resolução de clientes e cálculo de limite desacoplada de frameworks de transporte.
