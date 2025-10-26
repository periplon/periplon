# DSL Workflow Examples - Summary

## Overview

This directory contains **50 comprehensive DSL workflow examples** demonstrating the full capabilities of the SDK across diverse domains and use cases.

## Statistics

- **Total Examples**: 50
- **Total Lines of YAML**: ~    4356
- **Average Lines per Example**: ~87
- **Domains Covered**: 10
- **Unique Patterns Demonstrated**: 20+

## Examples by Domain

### 1. Web Development (5 examples)
- `01_frontend_build.yaml` - Modern frontend build with parallel testing
- `02_fullstack_deployment.yaml` - Multi-stage production deployment
- `03_api_development.yaml` - RESTful API development lifecycle
- `04_ui_testing.yaml` - Comprehensive UI/UX testing across browsers
- `05_pwa_build.yaml` - Progressive Web App optimization

**Key Features**: Parallel execution, build optimization, multi-browser testing

### 2. Data Science & ML (5 examples)
- `06_etl_pipeline.yaml` - Extract-Transform-Load with exponential backoff
- `07_ml_training.yaml` - End-to-end ML training with fallback agents
- `08_data_analysis.yaml` - Statistical analysis and visualization
- `09_feature_engineering.yaml` - Parallel feature creation
- `10_model_deployment.yaml` - ML model deployment and monitoring

**Key Features**: Fallback agents, retry strategies, experiment tracking

### 3. DevOps & Infrastructure (5 examples)
- `11_cicd_pipeline.yaml` - CI/CD with tool constraints
- `12_infrastructure_provisioning.yaml` - Terraform infrastructure as code
- `13_container_orchestration.yaml` - Kubernetes deployment
- `14_monitoring_setup.yaml` - Observability stack setup
- `15_disaster_recovery.yaml` - Backup and recovery procedures

**Key Features**: Tool constraints, rollback strategies, hooks integration

### 4. Finance & Banking (5 examples)
- `16_trading_algorithm.yaml` - Automated trading with fallback
- `17_risk_assessment.yaml` - Portfolio risk analysis
- `18_fraud_detection.yaml` - Real-time fraud detection with combined error recovery
- `19_portfolio_analysis.yaml` - Investment portfolio optimization
- `20_payment_processing.yaml` - Secure payment processing with fallback

**Key Features**: Exponential backoff, fallback agents, compliance checks

### 5. Healthcare (5 examples)
- `21_patient_data.yaml` - HIPAA-compliant data processing
- `22_medical_imaging.yaml` - Medical image analysis with fallback
- `23_clinical_trial.yaml` - Clinical trial management
- `24_health_monitoring.yaml` - Real-time health monitoring
- `25_drug_discovery.yaml` - Pharmaceutical research pipeline

**Key Features**: Privacy compliance, tool constraints, quality assurance

### 6. E-commerce (5 examples)
- `26_order_processing.yaml` - Order fulfillment workflow
- `27_inventory_management.yaml` - Stock management and forecasting
- `28_recommendation_engine.yaml` - ML-powered recommendations with fallback
- `29_customer_analytics.yaml` - Customer segmentation and churn prediction
- `30_price_optimization.yaml` - Dynamic pricing algorithms

**Key Features**: Parallel processing, recommendation systems, analytics

### 7. Content & Media (5 examples)
- `31_content_generation.yaml` - SEO-optimized content creation
- `32_video_processing.yaml` - Video transcoding with fallback
- `33_podcast_production.yaml` - Audio production workflow
- `34_social_media.yaml` - Multi-platform social media management
- `35_news_aggregation.yaml` - News curation and summarization

**Key Features**: Media processing, multi-platform publishing, web scraping

### 8. Education (5 examples)
- `36_curriculum_development.yaml` - Course design workflow
- `37_student_assessment.yaml` - Automated grading system
- `38_learning_analytics.yaml` - Educational data analysis
- `39_course_content.yaml` - Educational content creation
- `40_adaptive_learning.yaml` - Personalized learning paths

**Key Features**: Educational workflows, personalization, analytics

### 9. Security (5 examples)
- `41_security_audit.yaml` - Comprehensive security assessment with tool constraints
- `42_penetration_testing.yaml` - Ethical hacking workflow
- `43_threat_detection.yaml` - Real-time threat monitoring with exponential backoff
- `44_compliance_checking.yaml` - Regulatory compliance validation
- `45_vulnerability_scanning.yaml` - Automated vulnerability detection with combined recovery

**Key Features**: Security tools, compliance checks, threat detection

### 10. Research & Analysis (5 examples)
- `46_literature_review.yaml` - Academic literature review
- `47_market_research.yaml` - Market analysis and insights
- `48_competitive_analysis.yaml` - Competitor intelligence
- `49_scientific_experiment.yaml` - Experimental research workflow
- `50_patent_analysis.yaml` - IP research and patent landscape

**Key Features**: Research workflows, data synthesis, reporting

## Feature Coverage Matrix

| Feature | Examples Using It |
|---------|-------------------|
| **Parallel Execution** | 1, 6, 11, 26, 31, 32, 44 |
| **Sequential Execution** | 2, 7, 12, 21, 36, 49 |
| **Exponential Backoff** | 6, 11, 18, 22, 43, 45 |
| **Fallback Agents** | 7, 16, 18, 20, 22, 28, 32, 41, 45 |
| **Combined Error Recovery** | 18, 22, 32, 41, 45 |
| **Hooks (Pre/Post)** | 1, 2, 11, 15, 21, 41, 46, 49 |
| **Stage Hooks** | 7, 14, 23, 34, 44, 47 |
| **Error Hooks** | 2, 18, 20, 32, 41, 43 |
| **Tool Constraints** | 11, 21, 41, 44 |
| **Permission Modes** | 1, 3, 5, 21, 41 |
| **Multi-Stage Workflows** | 2, 7, 12, 23, 47, 49 |
| **Complex Dependencies** | 6, 13, 19, 27, 39 |
| **Output Files** | Most examples specify outputs |
| **Priority Handling** | All examples use priority |
| **Retry Strategies** | 1, 4, 6, 11, 16, 18, 24, 43, 46, 50 |
| **MCP Integration** | 8, 28, 35, 46, 50 (WebSearch tool) |

## Complexity Levels

### Beginner (Simple, <10 tasks)
- 08, 24, 26, 31, 37

### Intermediate (10-15 tasks)
- 01, 04, 06, 11, 16, 21, 36, 38, 40

### Advanced (15-20 tasks)
- 02, 03, 07, 12, 18, 23, 28, 29, 47

### Expert (Complex orchestration)
- 13, 19, 41, 45, 49

## Running Examples

### Basic Execution
```bash
cargo run --bin dsl-executor -- -f examples/dsl_workflows/01_frontend_build.yaml -w build_and_deploy
```

### With State Persistence
```bash
cargo run --bin dsl-executor -- -f examples/dsl_workflows/07_ml_training.yaml -w train_model --state-dir ./workflow_states
```

### Resume from Checkpoint
```bash
cargo run --bin dsl-executor -- -f examples/dsl_workflows/11_cicd_pipeline.yaml -w ci_cd --resume
```

### Dry Run (Validation Only)
```bash
cargo run --bin dsl-executor -- -f examples/dsl_workflows/41_security_audit.yaml -w audit_security --validate-only
```

## Pattern Highlights

### Error Recovery Patterns
- **Simple Retry**: Examples 1, 4, 16, 24
- **Exponential Backoff**: Examples 6, 11, 18, 43, 45
- **Fallback Agents**: Examples 7, 16, 18, 20, 22, 28, 32, 41, 45
- **Combined (Retry + Backoff + Fallback)**: Examples 18, 22, 45

### Orchestration Patterns
- **Parallel Task Execution**: Examples 1, 6, 26, 31, 32, 44
- **Sequential Stages**: Examples 2, 7, 12, 36, 49
- **Conditional Dependencies**: Examples 6, 13, 19, 27

### Integration Patterns
- **Tool Constraints**: Examples 11, 21, 41, 44
- **Permission Management**: Examples 1, 3, 5, 21, 41
- **Hooks Integration**: Examples 1, 2, 7, 11, 23, 34, 44, 47, 49
- **MCP Servers**: Examples 8, 28, 35, 46, 50

## Best Practices Demonstrated

1. **Error Handling**: All examples implement appropriate error handling
2. **State Management**: Output files specified for stateful tasks
3. **Resource Optimization**: Parallel execution where beneficial
4. **Security**: Tool constraints and permission modes in sensitive domains
5. **Observability**: Hooks for logging and monitoring
6. **Resilience**: Retry strategies with exponential backoff
7. **Modularity**: Clear task separation and dependencies

## Testing Examples

To test a specific example:

```bash
# Validate YAML syntax
cargo run --bin dsl-executor -- -f examples/dsl_workflows/XX_example.yaml --validate-only

# Execute workflow
cargo run --bin dsl-executor -- -f examples/dsl_workflows/XX_example.yaml -w workflow_name

# Execute with verbose logging
RUST_LOG=debug cargo run --bin dsl-executor -- -f examples/dsl_workflows/XX_example.yaml -w workflow_name
```

## Documentation

Each example is self-documenting with:
- Clear task descriptions
- Agent role definitions
- Workflow stage explanations
- Appropriate tool selections
- Error handling strategies

## Contributing

When adding new examples:
1. Follow the numbering scheme (`51_...`, `52_...`, etc.)
2. Include domain prefix in filename
3. Document key features in comments
4. Test workflow validation
5. Update README.md and this summary

## License

These examples are provided under the same license as the SDK.

---

**Created**: 2025-10-18  
**Total Examples**: 50  
**Coverage**: 10 domains, 20+ patterns  
**Status**: Production Ready ✅
