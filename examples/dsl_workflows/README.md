# DSL Workflow Examples

This directory contains 50 diverse DSL workflow examples showcasing different domains, patterns, and features of the Agent SDK.

## Table of Contents

### Web Development (5 examples)
1. [Frontend Build Pipeline](01_frontend_build.yaml) - Modern frontend build with testing and deployment
2. [Full-Stack Deployment](02_fullstack_deployment.yaml) - Complete application deployment workflow
3. [API Development](03_api_development.yaml) - RESTful API development with testing
4. [UI Testing Pipeline](04_ui_testing.yaml) - Comprehensive UI/UX testing
5. [Progressive Web App](05_pwa_build.yaml) - PWA build and optimization

### Data Science & ML (5 examples)
6. [ETL Pipeline](06_etl_pipeline.yaml) - Extract, Transform, Load data processing
7. [ML Model Training](07_ml_training.yaml) - End-to-end model training workflow
8. [Data Analysis](08_data_analysis.yaml) - Statistical data analysis pipeline
9. [Feature Engineering](09_feature_engineering.yaml) - Advanced feature engineering
10. [Model Deployment](10_model_deployment.yaml) - ML model deployment and monitoring

### DevOps & Infrastructure (5 examples)
11. [CI/CD Pipeline](11_cicd_pipeline.yaml) - Continuous integration and deployment
12. [Infrastructure Provisioning](12_infrastructure_provisioning.yaml) - Cloud infrastructure setup
13. [Container Orchestration](13_container_orchestration.yaml) - Docker/Kubernetes workflow
14. [Monitoring Setup](14_monitoring_setup.yaml) - Application monitoring and alerting
15. [Disaster Recovery](15_disaster_recovery.yaml) - Backup and recovery procedures

### Finance & Banking (5 examples)
16. [Trading Algorithm](16_trading_algorithm.yaml) - Automated trading system
17. [Risk Assessment](17_risk_assessment.yaml) - Financial risk analysis
18. [Fraud Detection](18_fraud_detection.yaml) - Real-time fraud detection
19. [Portfolio Analysis](19_portfolio_analysis.yaml) - Investment portfolio optimization
20. [Payment Processing](20_payment_processing.yaml) - Secure payment workflow

### Healthcare (5 examples)
21. [Patient Data Processing](21_patient_data.yaml) - HIPAA-compliant data processing
22. [Medical Image Analysis](22_medical_imaging.yaml) - Diagnostic image analysis
23. [Clinical Trial Management](23_clinical_trial.yaml) - Research trial coordination
24. [Health Monitoring](24_health_monitoring.yaml) - Patient health tracking
25. [Drug Discovery](25_drug_discovery.yaml) - Pharmaceutical research pipeline

### E-commerce (5 examples)
26. [Order Processing](26_order_processing.yaml) - Order fulfillment workflow
27. [Inventory Management](27_inventory_management.yaml) - Stock management system
28. [Recommendation Engine](28_recommendation_engine.yaml) - Product recommendation system
29. [Customer Analytics](29_customer_analytics.yaml) - Customer behavior analysis
30. [Price Optimization](30_price_optimization.yaml) - Dynamic pricing system

### Content & Media (5 examples)
31. [Content Generation](31_content_generation.yaml) - Automated content creation
32. [Video Processing](32_video_processing.yaml) - Video editing and transcoding
33. [Podcast Production](33_podcast_production.yaml) - Audio content workflow
34. [Social Media Management](34_social_media.yaml) - Multi-platform social media
35. [News Aggregation](35_news_aggregation.yaml) - News collection and curation

### Education (5 examples)
36. [Curriculum Development](36_curriculum_development.yaml) - Course design workflow
37. [Student Assessment](37_student_assessment.yaml) - Automated grading system
38. [Learning Analytics](38_learning_analytics.yaml) - Educational data analysis
39. [Course Content Generation](39_course_content.yaml) - Educational content creation
40. [Adaptive Learning](40_adaptive_learning.yaml) - Personalized learning paths

### Security (5 examples)
41. [Security Audit](41_security_audit.yaml) - Comprehensive security assessment
42. [Penetration Testing](42_penetration_testing.yaml) - Security testing workflow
43. [Threat Detection](43_threat_detection.yaml) - Real-time threat monitoring
44. [Compliance Checking](44_compliance_checking.yaml) - Regulatory compliance audit
45. [Vulnerability Scanning](45_vulnerability_scanning.yaml) - Automated vulnerability detection

### Research & Analysis (5 examples)
46. [Literature Review](46_literature_review.yaml) - Academic research workflow
47. [Market Research](47_market_research.yaml) - Market analysis and insights
48. [Competitive Analysis](48_competitive_analysis.yaml) - Competitor intelligence
49. [Scientific Experiment](49_scientific_experiment.yaml) - Experimental research workflow
50. [Patent Analysis](50_patent_analysis.yaml) - Patent research and analysis

## Feature Showcase

### Error Handling Examples
- **Retry with Exponential Backoff**: Examples 6, 11, 18, 22, 43
- **Fallback Agents**: Examples 7, 16, 20, 28, 41
- **Combined Error Recovery**: Examples 18, 22, 32, 41, 45

### Execution Patterns
- **Parallel Execution**: Examples 1, 6, 11, 26, 31
- **Sequential Execution**: Examples 2, 7, 12, 21, 36
- **Mixed Mode**: Examples 3, 13, 19, 29, 47

### Hooks Usage
- **Pre/Post Workflow Hooks**: Examples 11, 15, 21, 41, 49
- **Stage Completion Hooks**: Examples 7, 14, 23, 34, 44
- **Error Hooks**: Examples 18, 20, 43, 45, 50

### Advanced Features
- **Multi-Stage Workflows**: Examples 2, 7, 12, 23, 47
- **Complex Dependencies**: Examples 6, 13, 19, 27, 39
- **Tool Constraints**: Examples 11, 21, 41, 43, 44
- **MCP Server Integration**: Examples 8, 28, 35, 46, 50

## Running Examples

Execute any workflow using the DSL executor:

```bash
cargo run --bin dsl-executor -- -f examples/dsl_workflows/01_frontend_build.yaml -w build_and_deploy
```

With state persistence:

```bash
cargo run --bin dsl-executor -- -f examples/dsl_workflows/07_ml_training.yaml -w train_model --state-dir ./workflow_states
```

Resume from checkpoint:

```bash
cargo run --bin dsl-executor -- -f examples/dsl_workflows/11_cicd_pipeline.yaml -w ci_cd --resume
```

## Complexity Levels

- **Beginner** (Simple workflows): Examples 1, 4, 8, 26, 31
- **Intermediate** (Multi-stage): Examples 6, 11, 16, 21, 36
- **Advanced** (Complex orchestration): Examples 7, 12, 18, 23, 47
- **Expert** (Full-featured): Examples 13, 19, 41, 45, 50

## Domain-Specific Patterns

### Web Development
- Build optimization
- Testing strategies
- Deployment patterns

### Data Science
- Data validation
- Model versioning
- Experiment tracking

### DevOps
- Infrastructure as Code
- Blue-green deployments
- Rollback strategies

### Finance
- Transaction validation
- Audit trails
- Compliance checks

### Healthcare
- Data privacy (HIPAA)
- Audit logging
- Quality assurance

## Contributing

To add more examples:
1. Follow the naming convention: `{number}_{description}.yaml`
2. Include comprehensive documentation
3. Showcase specific features or patterns
4. Test the workflow before submitting

## License

These examples are provided under the same license as the SDK.
