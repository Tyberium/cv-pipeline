# Dave Carroll

> **Authoring draft only** — not loaded by the pipeline. Merge into `raw_history_plain.json`, then `cargo run --bin encrypt`. Never commit real `CV_SECRET` or Firebase keys in this file.

**Data Engineer**

* d.carroll@gmx.com
* Portishead, North Somerset
* [linkedin.com/in/dave-carroll-9b8b06139](https://www.google.com/search?q=https://linkedin.com/in/dave-carroll-9b8b06139)
* [github.com/Tyberium](https://www.google.com/search?q=https://github.com/Tyberium)

---

## Professional Summary

I am a Data Engineer with nearly ten years of experience in the field, backed by a prior decade of work as a Sysadmin. Having migrated from block-level infrastructure to high-scale cloud platforms, I've designed and optimized systems across a diverse range of companies and navigated everything from highly regulated financial environments to unstructured open-source APIs. 

My career journey to Data Engineer started with managing Gluster storage clusters, NetApp data layers, and bare-metal Hadoop stacks, before progressing to early distributed systems like Storm and Solr on self-managed bare-metal cloud. Today, I leverage that deep "under-the-hood" systems knowledge to design, optimize, and refactor modern cloud data infrastructure across AWS, GCP, and Databricks. Because I fundamentally understand how hardware and low-level distributed systems process data, I build cloud abstractions and pipelines that are fast, exceptionally stable, and highly cost-efficient.

I bring a proven track record of establishing the engineering standards, data quality requirements, and governance practices that eliminate noise and ensure clean, trustworthy insights.

### Key Accomplishments & Technical Highlights

* **Cloud Cost Optimization:** Led an infrastructure optimization strategy that reduced the company’s **total aggregate AWS bill by 25%**, transforming data retention and processing efficiency into major enterprise-wide savings.
* **Data Lake Migration:** Refactored and migrated a complex, multi-tenanted Data Lake in strict alignment with AWS Well-Architected practices, successfully isolating it from shared infrastructure, code bottlenecks, and cross-tenant concerns.
* **AI-Accelerated Delivery:** Adopted agentic development workflows to drastically accelerate project delivery, compressing the multi-tenanted Data Lake migration timeline from **months to weeks**.
* **Team Working & Mentorship:** Focus on building a professional, fun, and collaborative engineering culture. Actively mentor team members through constructive code reviews and real-time pair-programming sessions via Slack/Tuple to unblock technical hurdles.

### Technical Skills Matrix

* **Cloud Platforms:** AWS (Expert), GCP (Proficient)
* **Languages:** Python (Expert), SQL (Expert), TypeScript (Familiar), Rust (Familiar)
* **Data Engineering:** Spark / PySpark (Expert), Prefect (Proficient), Databricks (Proficient), Kafka (Familiar), DuckDB (Familiar)
* **DevOps & CI/CD:** Terraform (Expert), GitHub Actions (Expert), Docker (Proficient)
* **Databases:** PostgreSQL (Proficient), Elasticsearch / Solr (Familiar)
* **Core Systems & Tools:** Git (Expert), Linux (Proficient)

### Interests & Off-Duty

When I’m not optimizing the "machine spirit" of a production data lake, you can find me building hobby projects over at battleplan.uk, painting my way through my miniatures backlog, or occasionally getting to roll some dice at the tabletop. At 12:30 sharp, my Chihuahua politely requests her daily "walkies" by either sitting directly on my keyboard or testing the structural integrity of my office chair, and it's always a pleasure getting out and about with her. I also enjoy ice-skating (if you can call my stumbling around the rink "skating") with my kids, and whenever the wind blows just right, taking the power kite out for a spin.

---

## Why I'm applying to PostHog

I came across PostHog a few months ago when I was asked to set up a pipeline inside it to export data to S3 (Where's the Hive-style Partition by default button?), and the retro-style website drew me in straight away. After a bit more poking around, I started to feel that the ethos, company style, and direction very much resonated with me. I'm drawn particularly to the autonomy that is granted to engineers, and the openness of approach — from open-sourcing software to publishing pay scales (seriously, if I was Judge Dredd, this would be the LAW). When I saw the position to join the Ingestion team, I immediately started searching the garden to find a hedgehog I could use to measure myself, see where I fit in. (No luck yet; the Chihuahua has chased them all off).

Elephant in the room: I'm pretty new to Rust. I started playing with it toward the end of last year. I'd come across it a few times, but it was after I installed Ruff (a Rust-based Python linter) and it linted the entire repo I was working on in around 0.3 seconds that I had that "Oh, so that's what they mean by 'it's faster than Python'" moment. It's a language I'd love to get more into, would mercilessly use Claude to help out with, and if I did get the job, I'd make sure to spend the interim/notice period phase getting up to speed so I turned up on day 1 ready to get stuck in.

I hope you enjoy my CV, available here: `docker run -p 8080:8080 -e CV_SECRET=<provided-on-request> ghcr.io/tyberium/cv:latest`

It's over-engineered, weird, and totally unnecessary by design. You may even recognize some of the event schemas.

---

## Pipeline page (frontend only)

> **Not loaded by the pipeline** — edit `PIPELINE_STORY` in `frontend/src/pages/PipelineHealth.tsx` (or paste from here).

**Title:** This CV is a living, breathing pipeline

**Lead:** What happened was…

**Body:**

You ran a single Docker command. Before the browser opened, a script played out in the terminal — decrypt, ingest, transform, bus, warehouse, serve. That wasn't a loading spinner. That was the CV.

A single Rust binary unlocks encrypted history and a profile photo with CV_SECRET, pulls live skills and availability from Firebase, normalises everything into PostHog-style capture events, runs them through Redpanda into DuckDB, then serves this site from the gold layer. The wrong secret and the container stops at decrypt; no plaintext ever leaves the image.

The About, Employment, Skills, and Education pages are not hard-coded React props. They are query results from tables this run has just written. The stats below are telemetry from fact_pipeline_run. The startup log at the bottom is the same narrative that scrolled past in your terminal.

I took the problem of "a CV" and totally over-engineered it — to deliberately make something a bit weird, and hopefully a bit more fun.

You can find the source code for this pipeline here: https://github.com/tyberium/cv-pipeline

---

## Employment History

### Data Engineer — Wealth Wizards

* October 2023 – Present *
* Own day-to-day pipeline health: maintenance, root-cause on failures, and data quality — with automation to cut repeat incidents and downtime.
* Improve and simplify end-to-end data models; tackle architectural pain points and keep cloud spend down.
* Set team standards for coding, pipeline patterns, and modern stack migration; built a data quality strategy and rule engine so people can find and trust data without asking engineering every time.
* Mentor and onboard engineers, run syncs, retros, and planning, and keep the team unblocked.
* Work with stakeholders to turn requirements into delivery, with clear lineage and governance.

**Key Projects**

* Designed and deployed critical data pipelines with zero downtime and no data loss, optimizing workloads to reduce costs and delivering a 25% reduction in the company's total AWS cloud bill.
* Re-architected and migrated a six-year-old multi-tenant Data Lake to a new AWS platform. Established standards and coding patterns, defined access controls, and re-pathed ingestion (API to EventBridge and Firehose) and ETL workloads (ingest pipelines and raw tables to gold tables and views). Fully CI-integrated deployments to dev and production with drift prevention.
* Broke down a billing data silo by untangling logic held across Excel spreadsheets, aligning definitions with the client and finance team, and automating reconciliation in a QuickSight dashboard shared by both parties — reducing billing queries to zero.

### Data Engineer — Heni

* September 2022 – May 2023 *
* Develop and deploy efficient batch and streaming pipelines to ingest Web3 & Blockchain transaction data to power an NFT marketplace.
* Work with data scientists and stakeholders to capture requirements, scale existing data infrastructure, and secure Personally Identifiable Information (PII).
* Source, clean, and ingest disparate data from internal and external sources to a standard and easily consumed format.
* Mentor Junior Data Engineers, deliver team-wide training, provide software reviews and feedback via GitHub, and run pair-programming sessions.

**Key Projects**

* Developed pipelines to ingest Blockchain (Ethereum & Palm) transaction data. Built the necessary CI/CD workflows on GitHub Actions to deploy to AWS-based Databricks and optimized clusters to balance performance vs. cost.
* Designed a team-wide framework for running unit tests of cloud-based services locally (Pytest, Docker), to expedite development and deployment processes.
* Combined AWS serverless technologies (S3, Lambda, Glue, RDS, IAM, and Athena) with Delta Lake and PySpark to ingest data from a mix of APIs and internal databases to a single data warehouse (PostgreSQL) encompassing all customer information.

### Data Engineer — Envelop Risk

* October 2020 – September 2022 *
* Defined data platform strategy with a vision to migrate all data architecture to Databricks on Google Cloud Platform.
* Led a cross-team collaboration to develop a shared Python library to provide a Data Access Object (DAO) wrapper for Delta Lake.
* Responsible for writing Agile stories and allocating work to team members.

**Key Projects**

* Built a cross-team consensus for data strategy across multiple teams before presenting the plans to the Chief Technology Officer.
* Provisioned a data platform that has empowered data scientists to work at scale while controlling costs. This enabled the team to retrain our core machine learning algorithms to better model cyber risk.
* Redesigned the data warehouse and moved to a Data Lakehouse, which helped to break down data silos and allowed for greater collaboration between teams.
* Delivered CI pipelines (using predominantly GitHub Actions and Terraform) to ensure that trained and tested ML models would be rapidly deployed to production.

### DevOps Data Engineer — Polecat Risk Intelligence

* October 2018 – June 2020 *
* Part of the team responsible for ETL pipelines built in Python and Apache NiFi, hosted on AWS, ingesting over 20 million documents per day from a range of semi-structured data sources.
* Answered client and colleague queries by running SQL queries against the data warehouse (MySQL) and NoSQL queries against the company’s cloud-hosted data lake (Solr, Elasticsearch).
* Maintained Storm/Kafka clusters to support real-time data ingestion.

**Key Projects**

* Led a project to standardize methodology for ETL data collection streams and migrated legacy data flows to the new design.
* Worked as part of a team to design and implement a cloud-hosted data lake (Elasticsearch) which halved monthly data reprocessing costs.

### Capacity and Performance Analyst — Hargreaves Lansdown

* July 2017 – October 2018 *
* Maintained ETL pipelines to extract data from internal (Oracle, Hive, MySQL) data stores and load to a data warehouse (MySQL) for reporting of KPIs.
* Standardized data pipelines into Python and introduced Apache NiFi to orchestrate workflows and ingestion tasks.
* Produced monthly visualizations and reports for the Chief Information Officer (Excel, PowerBI) extracting data from Hive, concisely summarizing system performance, capacity, and availability for core IT systems.

### Windows Infrastructure Engineer — Various

* March 2008 – July 2017 *

### Trainee ICT Teacher — Various

* September 2006 – March 2008 *

### Retail Manager — Various

* September 2003 – September 2006 *

---

## Education

| Year | Qualification |
| --- | --- |
| 2022 | Data Engineering with Databricks |
| 2022 | Fundamentals of the Databricks Lakehouse Platform |
| 2020 | Python Best Practices for Code Quality |
| 2020 | Interpreting Data with Statistical Models |
| 2019 | Applying Real-time Processing Using Apache Storm |
| 2019 | Enterprise Data Management |
| 2019 | Enterprise Data Modelling Getting Started |
| 2019 | Supervised Learning With scikit-learn |
| 2018 | Understanding Machine Learning with Python |
| 2017 | Python - Beyond the Basics |
| 2016 | Data Warehouse Concepts, Design and Data Integration |
| 2016 | Database Management Essentials |
| 2016 | SQL |
| 2015 | NetApp ONTAP 8 |
| 2014 | ITILv3 Foundation |
| 2013 | Microsoft Certified Technology Specialist |
| 2012 | CISCO CCENT |
| 2007 | Diploma in IT — Liverpool John Moores University |
| 2003 | BSc Genetic Engineering — The University of Leicester |
| 2000 | GCSE/A-Levels — Formby High School |

---

## Affiliations & Interests

**Data Bristol** · Kiting · Southwest Amateur Gamers · Reading · DIY · PC/PS5 Gaming · Warhammer · Dog Training