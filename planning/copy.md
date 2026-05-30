# Dave Carroll

> **Authoring draft only** — not loaded by the pipeline. Merge into `raw_history_plain.json`, then `cargo run --bin encrypt`. Never commit real `CV_SECRET` or Firebase keys in this file.

**Data Engineer**

* d.carroll@gmx.com
* Portishead, North Somerset
* [linkedin.com/in/dave-carroll-9b8b06139](https://www.google.com/search?q=https://linkedin.com/in/dave-carroll-9b8b06139)
* [github.com/Tyberium](https://www.google.com/search?q=https://github.com/Tyberium)

---

## Professional Summary

I am a Data Engineer with nearly ten years of experience in the field, backed by a prior decade of work as a Sysadmin. Having migrated from block-level infrastructure to high-scale cloud platforms, I've designed and optimised systems across a diverse range of companies and navigated everything from highly regulated financial environments to unstructured open-source APIs. 

My career journey to Data Engineer started with managing Gluster storage clusters, NetApp data layers, and bare-metal Hadoop stacks, before progressing to early distributed systems like Storm and Solr on self-managed bare-metal cloud. Today, I leverage that deep "under-the-hood" systems knowledge to design, optimise and refactor modern cloud data infrastructure across AWS, GCP and Databricks. Because I fundamentally understand how hardware and low-level distributed systems process data, I build cloud abstractions and pipelines that are fast, stable, and cost-efficient.

I bring a proven track record of establishing the engineering standards, data quality requirements and governance practices that eliminate noise and ensure clean, trustworthy insights.

### Key Accomplishments & Technical Highlights

* **Cloud Cost Optimisation:** Led an infrastructure optimisation strategy that reduced the company’s **total aggregate AWS bill by 25%**, transforming data retention and processing efficiency into major enterprise-wide savings.
* **Data Lake Migration:** Refactored and migrated a complex, multi-tenanted Data Lake in strict alignment with AWS Well-Architected practices, successfully isolating it from shared infrastructure, code bottlenecks and cross-tenant concerns.
* **AI-Accelerated Delivery:** Adopted agentic development workflows to drastically accelerate project delivery, compressing the multi-tenanted Data Lake migration timeline from **months to weeks**.
* **Team Working & Mentorship:** Focus on building a professional, fun and collaborative engineering culture. Actively mentor team members through constructive code reviews and real-time pair-programming sessions via Slack/Tuple to unblock technical hurdles.

### Technical Skills Matrix

* **Cloud Platforms:** AWS (Expert), GCP (Proficient)
* **Languages:** Python (Expert), SQL (Expert), TypeScript (Familiar), Rust (Familiar)
* **Data Engineering:** Spark / PySpark (Expert), Prefect (Proficient), Databricks (Proficient), Kafka (Proficient), DuckDB (Familiar)
* **DevOps & CI/CD:** Terraform (Expert), GitHub Actions (Expert), Docker (Proficient)
* **Databases:** PostgreSQL (Proficient), Elasticsearch / Solr (Familiar)
* **Core Systems & Tools:** Git (Expert), Linux (Proficient)

### Interests & Off-Duty

When I’m not optimising the "machine spirit" of a production data lake, you can find me building hobby projects over at http://battleplan.uk, painting my way through my miniatures backlog, or occasionally getting to roll some dice at the tabletop. At 12:30 sharp, my Chihuahua politely requests her daily "walkies" by either sitting directly on my keyboard or testing the structural integrity of my office chair and it's always a pleasure getting out and about with her. I also enjoy ice-skating (if you can call my stumbling around the rink "skating") with my kids, playing pool, and whenever the wind blows just right, taking the kite out for a spin.

---

## Why I'm applying to PostHog

I was recently introduced to PostHog as part of a pipeline project. As with any new tool, I headed over to the site to familiarise myself with it. I was instantly hooked by the retro-PC design — it immediately felt like something a bit special. The thought "I wonder what this is like as a place to work" lodged itself firmly in my head.

Once off work's VPN, I had a proper dig around your Product Engineer Handbook and company pages. The more I read, the more I knew I wanted to work with you. I'm incredibly impressed by the radical transparency about well… everything, your focus on solving real user problems at speed, and your explicit "No assholes" policy.

When I saw you were recruiting for a Backend Engineer on the Ingestion team, I knew I had to apply. I also started searching the garden for a hedgehog to measure myself against — no luck yet; the Chihuahua has chased them all off.

In the PostHog spirit of being delightfully weird, this site is not a static CV — it is a full living ingestion pipeline built with Rust, TypeScript, and a Redpanda/Kafka messaging layer that borrows design patterns straight from PostHog's own codebase to serve my profile locally. Spin it up from your terminal:

docker run -p 8080:8080 -e CV_SECRET=<provided-on-request> ghcr.io/tyberium/cv:latest

Source code: https://github.com/Tyberium/cv-pipeline

You will need Docker to run it, but I am willing to bet that as ingestion engineers you have crossed paths with a container or two before. It is over-engineered, weird, and totally unnecessary by design. You may even recognise some of the event schemas.

I am farily new to Rust, I started playing around with it towards the end of last year. I had come across it a few times, but it was after I installed Ruff (a Rust-based Python linter) and it linted the entire repo I was working on in around 0.3 seconds that I had that "Oh, so that is what they mean by faster than Python" moment. It is a language I would love to get more into, would  use Claude to help out with, and if I did get the job I would spend the interim getting up to speed so I turned up on day one ready to get stuck in.

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
* Mentor and onboard engineers, run retros and planning sessions and act as go-to for unblocking engineers and figuring out complex code issues.
* Work with stakeholders to turn requirements into delivery, with clear data lineage and governance.

**Key Projects**

* Designed and deployed critical data pipelines with zero downtime, delivering a 25% reduction in the company's total AWS cloud bill.
* Re-architected and migrated a six-year-old multi-tenant Data Lake to a new AWS platform with full CI-integration and access control.
* Broke down a billing data silo by automating reconciliation in a QuickSight dashboard shared by client and finance, reducing billing queries to zero.

### Data Engineer — Heni

* September 2022 – May 2023 *
* Developed and deployed efficient batch and streaming pipelines to ingest Web3 and Blockchain transaction data to power an NFT marketplace.
* Worked with data scientists and stakeholders to capture requirements, scale existing data infrastructure, and secure Personally Identifiable Information (PII).
* Sourced, cleaned, and ingested disparate data from internal and external sources to a standard and easily consumed format.
* Mentored junior data engineers, delivered team-wide training, provided software reviews via GitHub, and ran pair-programming sessions.

**Key Projects**

* Developed pipelines to ingest Blockchain (Ethereum and Palm) transaction data with CI/CD on GitHub Actions to AWS Databricks, optimising clusters for performance vs cost.
* Designed a team-wide framework for running unit tests of cloud-based services locally (Pytest, Docker).
* Combined AWS serverless (S3, Lambda, Glue, RDS, IAM, Athena) with Delta Lake and PySpark to ingest APIs and internal databases into PostgreSQL.

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

American Pool · Kiting · Southwest Amateur Gamers · Reading · DIY · PC/PS5 Gaming · Warhammer 40K · Dog Training