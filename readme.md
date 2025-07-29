# 🦀 Rinha de Backend - Samuel Panzera 🦀

Este projeto é a minha participação na Rinha de Backend! O foco principal foi construir uma API de pagamentos com a **máxima performance** possível, explorando tecnologias e arquiteturas que otimizam a velocidade e o uso de recursos ao extremo.

## 🎯 O Intuito do Projeto

A meta era clara: criar a API mais rápida e eficiente possível para o desafio. Para isso, tomei algumas decisões técnicas ousadas, mas que se mostraram cruciais para atingir os objetivos de performance no contexto da "rinha".

## 🛠️ Stacks e Tecnologias Utilizadas

-   **Linguagem:** 🦀 **Rust** - Escolhido pela sua performance incomparável, segurança de memória e capacidades de concorrência de baixo nível.
    
-   **Framework Web:** ⚙️ **May/May-MiniHttp** - Um framework Rust minimalista que utiliza corrotinas (green threads), permitindo um altíssimo grau de concorrência com um custo muito baixo de recursos.
    
-   **Banco de Dados:** 🗄️ **SQLite (Embedded)** - O banco de dados roda _dentro_ da própria aplicação, uma decisão chave para eliminar a latência de rede e economizar recursos.
    
-   **Load Balancer:** **Nginx** - Para distribuir a carga de forma eficiente entre duas instâncias da API, garantindo alta disponibilidade e escalabilidade horizontal.
    
-   **Containerização:** 🐳 **Docker & Docker Compose** - Para facilitar a configuração, o deploy e a execução de todo o ambiente de forma consistente.
    

## 🏗️ Decisões Técnicas e Arquitetura

<img width="720" alt="image" src="https://github.com/user-attachments/assets/2d63a842-5bcc-433a-8ec2-67180923313c" />


A decisão mais impactante na arquitetura deste projeto foi a escolha de um **banco de dados _embedded_**.



### Por que um Banco de Dados Dentro da API?

-   ⚡ **Performance Extrema:** A comunicação com o SQLite é feita através de chamadas de função diretas na memória. Isso elimina completamente a latência de rede (network overhead) que existiria ao se conectar com um serviço de banco de dados externo (como PostgreSQL ou MySQL). Em um desafio focado em microssegundos, essa diferença é brutal.
    
-   📉 **Economia de Recursos:** Manter o banco de dados na mesma instância da aplicação reduz drasticamente o consumo de memória e CPU, já que não há um processo separado de banco de dados para gerenciar.
    

> **⚠️⚠️⚠️ AVISO IMPORTANTE⚠️⚠️⚠️:** Embora essa abordagem seja perfeita para o cenário competitivo da Rinha de Backend, **JAMAIS deve ser feito para projetos em produção!** Em um ambiente real, um banco de dados separado (como PostgreSQL) oferece vantagens críticas de escalabilidade, segurança, manutenibilidade e resiliência que são, na maioria das vezes, mais importantes do que a performance bruta obtida com um banco _embedded_.

## ✅ Checklist de Funcionalidades

### Prontas para a Batalha!

-   [x] 🚀 API para processamento de pagamentos (`POST /payments`)
    
-   [x] 📊 API para resumo de pagamentos (`GET /payments-summary`)
    
-   [x] 🗄️ Estrutura e otimizações do banco de dados SQLite com `PRAGMA`
    
-   [x] ⚖️ Configuração do Load Balancer com Nginx para duas instâncias
    
-   [x] 🐳 Containerização completa com Docker e Docker Compose
    

### Próximos Passos (O que falta implementar)

-   [ ] 📅 Implementar filtros de data (`from` e `to`) no endpoint de resumo
    
-   [ ] 🔄 Implementar a comunicação entre as instâncias para consolidar o resumo de pagamentos
    
-   [ ] 🧠 Desenvolver a lógica de seleção do processador de pagamento (verificação de `service-health`)
    
-   [ ] 📤 Implementar a comunicação real com os processadores de pagamento externos
    

## 🚀 Como Executar o Projeto

Para botar a rinha pra rodar, você só precisa do Docker e do Docker Compose instalados.

1.  Clone este repositório.
    
2.  Abra o terminal na pasta raiz do projeto.
    
3.  Execute o comando mágico:
    
    Bash
    
    ```
    docker-compose up
    
    ```
    

Isso irá subir todo o ambiente: as duas instâncias da API e o Nginx atuando como load balancer.
