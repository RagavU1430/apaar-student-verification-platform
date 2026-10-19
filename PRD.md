## PRODUCTREQUIREMENTSDOCUMENT

## APAAR-BasedStudentData &VerificationPlatform Management

Rust-firstbackendimplementationforAICTE-approvedinstitutions

| Documentfield | Value 26263 |
| --- | --- |
| ProblemStatementID |   |
| Organization/Department | AICTE/RegulationBureau |
| Category/Theme | Software/SmartAutomation |
| Documenttype | ProductRequirementsDocument(PRD) |
| Version/status | 1.0—Proposedspecificationforplanning |
| Date | 19October2026 |
| Intendedteam | 1– 2developers;2– 7dayprototypewindow |

## IMPORTANTSCOPENOTE

ThisPRDisaproposedproductdesignderivedfromthesuppliedproblemstatement. It isnotanofficial AICTEspecification, legalopinion,orconfirmationofaccesstoliveAPAAR/governmentAPIs.Those

interfaces,rules,andapprovalsmustbevalidatedwithauthorizedstakeholders.


## Documentcontrol andhowtousethisPRD

Thisdocumentdefinestheproblem,users,requirements,workflows,datamodel,Rustarchitecture,securityexpectations, deliveryplan,acceptancecriteria,risksandopendecisionsforaprototypethatcanlaterevolvetowardaproductionsystem.

- UsetheP0requirementsasthesmallestcoherenthackathonMVP.

- TreatP1itemsasvaluableextensionsaftertheprimaryworkflowisstable.

- Treatallexternal identityverification,policy-specificintakecalculations,andregulatoryintegrationsasgateduntilofficially confirmed.

- Donotuserealstudentpersonaldatainthehackathondemo.Usesyntheticrecords.

| Priority | Meaning |
| --- | --- |
| P0—Musthave | Requiredtodemonstratethecoreworkflowandthemostimportantproblem-statementrequirements. |
| P1—Shouldhave | High-valuecapabilitythatimprovestrust,usabilityorevaluatorconfidence. |
| P2—Couldhave | Optionalextension;donotjeopardizethecoreMVPtobuildit. |

## 1.Executivesummary

AICTEreceivesstudent-relatedinformationthroughdifferentsubmissionmechanisms.Thesuppliedproblemstatement explainsthatPGDMinstitutionsprovidedetailedstudentdataannually,whileotherAICTE-approvedinstitutionsprimarily submitaggregateenrolmentinformationduringapproval.Thismakescomplete,authentic,comparableandverifiable student-levelrecordsdifficulttomaintaincentrally.

Theproposedplatformprovidesastandardizedpathforinstitutionstosubmitandupdatestudent-levelrecordslinkedto APAARID,validatetherecords,detectduplicatesandinconsistencies,compareactualenrolmentwithapprovedintake,and makeauthorizedinformationavailablethroughdashboards,reportsandverificationworkflows.

ThesystemshouldbedesignedasanAPI-firstregulatorydataplatform.Rustisproposedforthebackendservicesbecauseits typesystem,memorysafetyandexpliciterrorhandlingareuseful forbuildingdependableservices.Rustdoesnotitself guaranteeapplicationsecurityorregulatorycompliance;thosedependonarchitecture,controls, testingandgovernance.

## Productvision

Createareliable,privacy-aware,interoperablestudent-recordecosystemthathelpsauthorizedstakeholdersunderstandwhat wassubmitted,whatwasvalidated,what remainsunresolved,andwhichrecordscanbeverifiedforadefinedpurpose.

## Productoutcome

Anauthorizedinstitutioncanuploadabatch,receiverecord-levelvalidationresults,resolveissues,andsubmitaccepted records.AnAICTEadministratorcanmonitorenrolmentandexceptiontrendsacrossinstitutions.Anauthorizedverifiercan checkalimitedrecordstatuswithoutreceivingunrestrictedaccesstothecentralrepository.

## 2.Problemstatement,needandeffects

## 2.1Currentproblem

- Submissionpracticesdifferbyinstitutiontypeandprocess.

- Aggregateenrolmenttotalsdonotprovidethestudent-leveldetailneededforindividualrecordchecks.

- Missing,malformed,duplicatedorinconsistentidentifierscanunderminethequalityofacentralrepository.

- Admissionsmayneedtobecomparedwithapprovedintakeatthecorrectinstitution/programme/academic-yearlevel.

- Authorizedstakeholdersneedatrustworthywaytoverifypermittedinformationwithoutexposingallstudentdetails.

- Regulatoryreportingismoreusefulwhendatadefinitionsandvalidationrulesareconsistentacrossinstitutions.


## 2.2Whoandwhat isaffected?

| Stakeholder | Effect of the current gap | Desired outcome |
| --- | --- | --- |
| AICTE/Regulation Bureau | Limited consistent visibility into student-level records and intake compliance. | Central monitoring, exception queues, trustworthy reports and trend analysis. |
| Institution administrators | Repeated formatting, submission corrections and uncertainty about rejected records. | Clear upload templates, actionable errors, correction workflow and submission status. |
| Students | Potential identity mismatch, duplicate records or inaccurate institutional data. | Accurate records, correction route, limited disclosure and accountable access. |
| Authorized verifiers | Difficulty establishing whether a student record is valid and current. | Purpose-limited verification result with clear provenance and timestamp. |
| Degree-awarding bodies | Need to associate student IDs and APAAR IDs on certificates where required. | Defined field and display/integration support, subject to approved rules. |
| Policy and accreditation teams | Inconsistent data may limit comparison across programmes and institutions. | Comparable, traceable and appropriately aggregated reports. |

## 2.3Whytheplatformisneeded

Theplatformshouldturndatasubmissionintoacontrolleddata-qualityandverificationprocess. Itsvalueisnotjuststoring records: itmustpreventavoidableerrors,makeunresolvedissuesvisible,supporthumanreviewofambiguouscases,and preserveatraceablehistoryofchanges.

## 2.4Expectedimpact

- Improvedcompletenessandconsistencyofstudent-levelrecords.

- Earlierdetectionofpossibleduplicateactiveenrolmentsandcapacityviolations.

- Reducedmanualeffort toidentifyerrorsinlargesubmissions.

- Fasteraccesstoauthorizedverificationoutcomes.

- Moreconsistentevidenceforregulatorymonitoringandpolicyanalysis.

Impactshouldbemeasuredusingoperationalmetricssuchasvalidationerrorrate,medianbatch-processingtime,percentage ofrecordswithresolvedissues,numberofintakeexceptionsreviewed,andverificationresponsetime.Theprototypeshould notclaimthatithasreducedfraudorimprovednationaldataqualitywithoutmeasuredevidence.

## 3.Productgoals,non-goalsandsuccessmetrics

## 3.1Goals

- Provideasecure,standardizedmechanismforinstitution-levelstudent-datauploadandupdates.

- UseAPAARIDastheproposedacademicidentitykeywhilepreservinginstitution-specificstudentIDsandadmission history.

- Validatemissing/invalididentifiers,duplicaterecords, inconsistentdataandintakeexceptions.

- Providerole-appropriateinstitutionandAICTEdashboards.

- Supportpurpose-limitedverificationforauthorizedstakeholders.

- Maintainauditablehistoryofsubmissions,corrections,reviewsandverificationrequests.

- ProvidereportandAPIcapabilitiessuitableforfutureintegration.

## 3.2Non-goalsforthe2– 7dayprototype

- ReplacingAPAAR,DigiLocker,AICTEorinstitutionalMIS/ERPsystems.

- Claiminglivegovernmentverificationwithoutanapprovedintegrationandcredentials.

- Buildinganationwideproductiondeploymentorcompletingaformalsecuritycertification.

- Makingautomatedfinaldeterminationsof fraud,eligibility,admissionlegalityorstudentmisconduct.


- UsingAI/MLasasubstituteforexplicitregulatoryrulesorhumanreview.

- Collectingunnecessarypersonal information, identitydocumentsorbiometricdata.

## 3.3Successmetrics

| Metric | Prototype target (proposed) | Measurement |
| --- | --- | --- |
| Duplicate detection | Detect exact APAAR-ID collisions in the seeded dataset. | Automated test suite with known valid/invalid sample records. |
| Intake check | Test same-upload and cross-institution cases. | Compare calculated counts to known fixtures. |
| Batch validation correctness | All seeded test cases produce expected results. | Correctly flag configured cases exceeding applicable demo intake. |
| Traceability | Every create/update/review action creates an audit event. | Integration test and audit-log inspection. |
| Access control | Unauthorized role cannot access restricted endpoints. | Negative authorization tests. |
| Usability | User can upload a batch and understand failed rows without developer help. | Observed task walkthrough. |
| Performance | Process 1,000 synthetic rows in under 10 seconds on the chosen demo environment. | Repeatable local benchmark; not a production SLA. |

Thesetargetsareinitialengineeringgoalsfortheprototype,notofficialAICTEservice-levelrequirements.Revisitthemafter defininghostingresources,datavolumeandoperatingconstraints.

## 4.Users,rolesandpermissions

| Role | Primary responsibilities | Default access boundaries |
| --- | --- | --- |
| Institution Admin | Manage institution profile and approved programmes; upload/update own institution records; review validation results. | Only institution-scoped records; cannot administer other institutions. |
| Institution Data Operator | Prepare batches, submit records and fix validation issues. | Institution-scoped access; no global policy or user administration. |
| AICTE Regulator / Admin | Monitor institutions, review exceptions, configure authorized rules, produce regulatory reports. | Access limited by assigned regulatory role and audited purpose. |
| Authorized Verifier | Request a defined verification result for an identified record. | Only minimum approved fields; no general student search or bulk export by default. |
| Auditor / Read-only Reviewer | Inspect relevant submission and access history. | Read-only access to assigned audit scope. |
| Student (future phase) | View permitted record status and request correction through a defined workflow. | Only own record after a reliable identity/authorization method exists. |
| System Operator | Manage deployment, service health and backups. | Operational access should not automatically grant unrestricted business-data access. |

Authorizationmustbeenforcedinthebackendoneveryrelevantrequest.HidingabuttoninReactisnotanaccess-control mechanism. Institutionscopemustcomefromtheauthenticateduser'strustedserver-sideidentity,notfromanuntrusted institution_idsuppliedbytheclient.

## 5.Coreuserjourneys

## 5.1Institutionuploadsastudentbatch


- 1. Usersignsinandselectstheinstitution/programme/academicyeartheyareauthorizedtomanage.

- 2. UserdownloadsorfollowstheversionedCSVtemplateanduploadsafile.

- 3. APIchecksfiletype,size,encoding,requiredcolumnsandrowcountlimits.

- 4. Systemcreatesabatchrecordandvalidateseachrowwithoutsilentlydiscardingerrors.

- 5. UIshowscountsforacceptedcandidates,warnings,rejectedrowsandduplicatecandidates.

- 6. Userdownloadsarow-levelerrorreport,correctsthesourcefileandresubmitsorusesanauthorizedcorrectionworkflow.

- 7. Acceptedrecordsarecommittedtransactionallywithanauditeventandabatchsummary.

## 5.2AICTEreviewsanintakeexception

- 8. Dashboarddisplaysapprovedintakeandactualadmissionsusingtheconfiguredcountingrule.

- 9. Anexceptioniscreatedwhentheapplicablelimitisexceededorthedataisinconsistent.

- 10. Revieweropenstheexceptionandseesthecalculation,sourcerecordsandruleversion.

- 11. Reviewerrecordsanoutcomesuchasconfirmedissue,datacorrectionrequested,ordismissedwithreason.

- 12. Thesystempreservestheoriginalalertandreviewhistory; itdoesnoterasetheevidencewhenstatuschanges.

## 5.3Authorizedstakeholderverifiesarecord

- 13. Verifierauthenticatesandselectsanapprovedverificationpurpose.

- 14. Thesystemcheckstheverifier'srole,scope,purposeandpermitteddatafields.

- 15. Thesystemreturnsalimitedresultsuchasverifiedagainsttheplatform'srecords,needsreview,ornotfound.

- 16. Theresponsestatesthesourceandtimestamp;itdoesnotimplyliveAPAARauthorityunlessthatsourcewasactually consulted.

- 17. Therequestanddisclosedfieldsarerecordedintheaudittrail.

## 5.4Studentrecordcorrection

Acorrectionshouldcreateanewversionoratraceablechangeeventratherthanoverwritingtheonlyhistoricalcopy. Dependingonthefieldandpolicy,thecorrectionmayrequireinstitutionalapprovalorregulatorreview.Thesystemshould preservewhorequested,approvedandappliedthechange.

## 6.Functional requirements

| ID | Requirement | Priority | Acceptance criteria |
| --- | --- | --- | --- |
| FR-01 | Authentication: users can sign in and obtain a server-validated session/token. | P0 | Invalid credentials fail; protected endpoints reject unauthenticated calls. |
| FR-02 | Role-based access and institution scoping. | P0 | Cross-institution access attempts are rejected and logged. |
| FR-03 | Institution and programme configuration. | P0 | Authorized admin can configure institution, programme, academic year and intake fixture. |
| FR-04 | Bulk upload through a documented CSV template. | P0 | Valid file creates a batch; malformed file returns actionable errors. |
| FR-05 | Row-level validation report. | P0 | Each row receives accepted/rejected/warning status and one or more reason codes. |
| FR-06 | APAAR field validation. | P0 | Missing/format-invalid values are flagged. Format checks are explicitly not identity verification. |
| FR-07 | Duplicate detection within a batch and across institution records. | P0 | Exact collisions are detected; ambiguous cases are flagged for review. |
| FR-08 | Admission and intake comparison. | P0 | Counts are calculated by institution/programme/year and applicable seat rules. |
| FR-09 | Central student and admission registry. | P0 | Accepted records can be searched by authorized users and retain admission history. |
| FR-10 | Record correction / review workflow. | P0 | Issue can be assigned, commented on and resolved without deleting history. |
| FR-11 | Institution dashboard. | P0 | Shows own accepted records, issues, batch status and configured intake utilization. |
| FR-12 | AICTE dashboard. | P0 | Shows aggregate institution/programme metrics and prioritized exceptions. |
| FR-13 | Audit trail. | P0 | Upload, edit, review, role change and verification actions produce attributable events. |
| FR-14 | Purpose-limited verification endpoint. | P1 | Verifier receives only allowed fields; request is authenticated and audited. |
| FR-15 | Report export. | P1 | Authorized user exports filtered aggregates or row-level issues within permitted scope. |
| FR-16 | REST integration API and versioning. | P1 | API contract is documented and uses stable error codes and schema versions. |
| FR-17 | QR-based verification link. | P2 | QR resolves to a protected or deliberately minimal verification page; no secret data is embedded. |
| FR-18 | Explainable anomaly score. | P2 | Score is derived from documented rules and shows the evidence; never makes an automatic fraud verdict. |
| FR-19 | Certificate display support for APAAR ID plus student ID. | P2 | Field mapping and output format are demonstrable; official issuance integration remains separate. |

## 7.Datavalidationandregulatoryrules

## 7.1Validationclasses

| Class Structural | Examples Missingrequiredcolumn, invaliddate,unsupportedfile, |   | Systembehavior Rejectfileor rowwithaclearerrorcode. |
| --- | --- | --- | --- |
| Field-level | oversizedupload. MissingAPAARID,emptystudentID,malformed |   | Rejectormarkasneedingcorrectionaccordingtoconfigured |
| Uniqueness | programmecode. SameAPAARIDrepeatedinabatchorlinkedtomultiple |   | rule. Detectexactduplicate;cross-institutionconflictbecomesa |
| Referential | activeadmissions. Programmeorinstitutiondoesnotexist;academicyear |   | reviewableflag. Rejectorquarantineuntilmasterdataiscorrected. |
| Capacity | notconfigured. Countexceedsapprovedintakefora |   | Createanintakeexceptionwithcalculationandpolicy-rule |
| Consistency | programme/year/seatcategory. Admissiondateoutsideallowedperiod;categorytotals |   | version. Flagwiththespecificruleandrelevantvalues. |
| Anomaly | donotreconcile. Unusualchangefrompriorsubmissionorunusuallyhigh |   | Generateexplainableriskindicatorforhumanreview. |
|   | correctionrate. |   |   |

## 7.2Intakecountingmustbepolicy-configurable

Donothard-codeasingleuniversal formulaforeveryseatcategory.Theapplicabletreatmentofsupernumeraryseats,NRI category,OCIstatusandforeign-nationalstatusmustbeestablishedfromofficialAICTErulesandtherelevantacademic-year approval.Storeapprovedintakeandcountingruleswithaneffectivedate/version.


Forademo,defineanexplicitsamplerule:countacceptedactiveadmissionsbyinstitution+programme+academicyear, then comparewiththeconfiguredregularintakeandseparatelyconfiguredapprovedsupernumerarycapacity.Label thisasademo ruleuntilvalidated.

## 7.3APAARvalidationlevels

- Level1:required/presentcheck.

- Level2:formatandlengthchecksonlyifanofficial formatspecificationhasbeenconfirmed.

- Level3:authoritativeliveverificationonlythroughanauthorizedAPI/processwithapprovedcredentials.

TheUIandAPImustdistinguishtheselevels.Avaluepassingformatchecksmustneverbelabelled'APAARverified'.

## 7.4Duplicatelogicandexceptions

- SameAPAARIDrepeatedinthesamebatch: flagasaduplicatecandidateandidentifyrownumbers.

- SameAPAARIDlinkedtomorethanoneactiveadmission:raiseacross-institutionconflictforreview.

- Historical,withdrawn,completedortransferredadmissionsmustberepresentedwithstatusanddatessolegitimatecases arenotautomaticallytreatedasfraud.

- Donotexposeastudent'sinstitutionhistorytoadifferentinstitutionunlessthatdisclosureisexplicitlyauthorized.

- Keepthedecision,evidenceandrevieweridentityfordismissedorconfirmedflags.

## 8.Datamodelproposal

| Entity Institution | Importantfields id,official_code,name,status,created_at | Notes Official identifiersshouldbevalidatedagainst approvedmasterdata. |
| --- | --- | --- |
| Programme | id, institution_id,code,name, level,status id, institution_id,programme_id,academic_year,regular_intake, | Institution-scopedprogrammecatalogue. Capacitymustbetraceabletoanapproval |
| ApprovedIntake | supernumerary_rules,source_reference,effective_version | sourceandeffectiveperiod. |
| StudentIdentity | id,apaar_id, institution_student_id(wherescoped),minimal | Avoidstoringunnecessarypersonaldata; identityuniquenessstrategyrequirespolicy |
|   | permittedattributes,created_at | review. |
| Admission SubmissionBatch ValidationIssue VerificationRequest AuditEvent User/Role | id,student_identity_id, institution_id,programme_id, academic_year,admission_date,status,seat_category, source_batch_id id, institution_id,submitted_by,file_hash,schema_version,status, counts,submitted_at id,batch_id,row_reference,entity_id,rule_code,severity,details, status,resolved_by,resolved_at id,requester_id,purpose,record_ref, fields_disclosed,outcome, requested_at id,actor_id,action,entity_type,entity_id,timestamp,correlation_id, safe_change_summary id, login_identifier,role,institution_scope,status,auth_metadata | Multiplehistoricaladmissionsmaybevalid. Storefilemetadata;avoidretainingrawfiles longerthannecessary. Detailsmustavoidunnecessarypersonaldata. Auditaccessanddatadisclosure. Append-onlyapplication-leveleventhistory; restrictandmonitoraccess. Neverstoreplaintextpasswords. |

## 8.1Databaseintegrity

- Useprimarykeysandforeignkeyswhereverthechosenrelationaldesignsupportsthem.

- Usedatabaseconstraintsforinvariantrulesthatmustholdunderconcurrentrequests.

- DefinehowanAPAARIDisnormalized,protectedandindexed;donotcasuallylogit.

- Usetransactionsforbatchcommitsandreview-statechanges.

- Useidempotencykeysorbatchfingerprintstopreventaccidentalduplicatesubmissions.


- StoretimestampsinUTCandrenderlocal timeintheUI.

- Defineretentionanddeletionpolicieswiththedatacontroller/authorizedinstitution;donotinventauniversalretention period.

## 9.Rust technologyandarchitecture

## 9.1Recommendedstack

| Concern Language HTTPserver Databaseaccess Serialization Validation Authentication Authorization Fileimport | Recommendation StableRusteditionsupportedbyteamtooling Axum+Tokio+Towermiddleware SQLx+PostgreSQL Serde+serde_json validatorcrateorexplicitdomainvalidation functions OIDCprovider ifavailable;otherwiseacarefully scopedprototypeauthimplementation Explicitroleandinstitution-scopechecksin servicelayer/middleware CSVcrate;XLSXsupportonlyiftimepermits | Reason/caveat Pincompiler/toolchainversionfor reproduciblebuilds. Asyncrequesthandlingandtypedextractors;keephandlersthin. Compile-timequerycheckingwhereconfigured;relational constraintsandtransactionsfitthisdomain. TypedJSONrequest/responsemodels. Businessrulesshouldbetestableandnotburiedinroutehandlers. Avoidhand-rollingproductionidentitymanagement.JWTsneed signature, issuer,audience,expiryandkeyvalidation. RBACalonemaybeinsufficient;record-levelscopemustalsobe enforced. CSViseasiertovalidate,testanddemoreliably. |
| --- | --- | --- |
| Migrations | SQLxmigrations | Schemachangesmustberepeatableandversion-controlled. AvoidloggingAPAARIDs,tokens,rawrowsorothersensitive |
| Observability | tracing+tracing-subscriber;structuredlogs | payloads. |
| APIdocumentation |   | OpenAPIgenerationormaintainedOpenAPIspec Contractsupportsfrontendandfuturesystemintegrations. |
| Frontend | React+TypeScript+Vite Docker,managedPostgreSQL,TLS-enabled | AseparatefrontendcanconsumetheRustRESTAPI. Usesecretsmanager/environmentinjection;nevercommit |
| Deployment | reverseproxy | secrets. |

## 9.2Logicalmodules

- api:routedefinitions,requestextraction,responsetypesandAPIversioning.

- auth: identity/sessionvalidation,rolechecksandinstitution-scopeenforcement.

- institutions: institution/programmeconfigurationandapproved-intakemasterdata.

- ingestion:uploadlimits, fileparsing,batchtrackingandidempotency.

- validation:field,uniqueness,referential, intakeandconsistencyrules.

- admissions:studentidentityandadmissionlifecycleoperations.

- verification:purpose-limitedlookupandfield-disclosurepolicy.

- reporting:aggregatemetrics, filteredreportsandexports.

- audit:structuredauditeventsandreviewhistory.

- infrastructure:databasepool,configuration,logging,healthchecksandexternaladapters.

## 9.3Architecturepattern

Forthisteamsize,useamodularmonolithratherthanmicroservices.OneRustservicecanownthedomainmodulesand transactionboundaries.Separatemodulesinternally,keepexternal integrationbehindanadaptertrait,andavoidintroducing messagebrokersordistributedinfrastructureunlessavalidatedrequirementdemandsthem.

## 9.4ExternalAPAARintegrationboundary


DefineatraitorinterfacesuchasAcademicIdentityVerifierwithamockimplementationfordevelopmentandaseparate productionadapteronlyaftertheofficialendpoint,authentication,permittedfields,errormodel,ratelimitsandapprovalprocess areknown.Themockmustvisiblyidentifyitselfassimulated.DonotfabricatealiveAPIresponse.

## 9.5Errorhandling

- Returnconsistentmachine-readableerrorcodes,safehuman-readablemessagesandarequest/correlationID.

- Use400forinvalidinput,401 forunauthenticated,403forunauthorized,404forinaccessible/not-foundresources accordingtodisclosurepolicy,409forconflicts,413foruploadtoolarge,422forrow-levelsemanticerrorswhen appropriate,and500forunexpectedfailures.

- NeverreturnSQLerrors,stacktraces,secretsorinternalinfrastructuredetailstoclients.

- Forbatchuploads,distinguishfile-levelfailurefromrow-levelvalidationissues.

## 10.APIrequirements

SuggestedRESTAPIsurface.Routenamesandpayloadsareproposed; finalizetheminanOpenAPIcontractbefore implementation.

| Methodandroute | Purpose | Access |
| --- | --- | --- |
|   |   | Publicendpointwithratelimiting; |
| POST/api/v1/auth/login | PrototypeloginorOIDCinitiation | preferablydelegateauthtoan |
| GET/api/v1/me |   | identityprovider. |
| GET/api/v1/institutions | Returncurrentuser,roleandpermittedinstitutionscope Listinstitutionsavailabletocurrentrole | Authenticated Scoped/authenticated |
| GET/api/v1/institutions/{id}/intakes | Readapprovedintakeconfiguration | Scoped/authenticated |
| POST/api/v1/batches | CreatebatchanduploadCSV | Institutionoperator/admin |
| GET/api/v1/batches/{id} | Getbatchstatusandaggregateresults | Owner institutionorregulator |
| GET/api/v1/batches/{id}/issues | Getrow-levelvalidationissues | Owner institutionorregulator |
| POST/api/v1/batches/{id}/commit | Commiteligiblerecordsaftervalidation | Authorizedinstitutionuser; |
| GET/api/v1/admissions | Search/filterrecordswithinpermittedscope | idempotent Scoped/authenticated |
| GET/api/v1/dashboard/institution GET/api/v1/dashboard/aicte | Institutionmetrics Cross-institutionaggregatemetrics | Institutionrole AICTErole |
| POST/api/v1/issues/{id}/resolve | Resolveordismissanissuewithreason | Assignedreviewer/authorizedrole |
| POST/api/v1/verifications GET/api/v1/reports/enrolment GET/health/liveand/health/ready | Requestpurpose-limitedrecordverification Getorexportauthorizedenrolmentreport Processanddependencyhealthchecks | Authorizedverifier Scoped/authenticated Operationalendpoint;nosensitive details |

## 10.1APIcontractrules

- Allprotectedendpointsrequireauthenticatedidentityandserver-sideauthorization.

- Validatequeryfiltersandpagination;cappagesizeandexportvolume.

- UseexplicitAPIversioningandstableerrorcodes.

- Setmaximumuploadsize,rowcount,processingtimeandrequestbodylimits.

- Userequest IDsforsupportandsafeauditcorrelation.

- Documentwhichfieldsmaycontainpersonaldataandwhicharereturnedtoeachrole.

- Rate-limitlogin,verificationandexpensiveexportendpoints.

- Useidempotencyordeduplicationsafeguardsforretriesandbatchcommits.


## 11.Frontendanduserexperiencerequirements

| Screen Sign-in Institutiondashboard Uploadwizard | Requiredcontent Loginstatus,safeerrormessagesandsessionhandling. Latestbatchstatus,accepted/rejected/warningcounts,ownintakeutilizationandunresolvedissues. Templatedownload, fileconstraints,uploadprogress,validationsummaryandnextactions. |
| --- | --- |
| Batchdetail | Rownumber,safestudentreference, issuecode,explanation,severityanddownloadableerrorreport. |
| Student/admissiondetail | Onlypermittedattributes,currentstatus,historyandauthorizedcorrectionactions. |
| AICTEdashboard Issuereview | Institution/programmefilters,aggregateenrolment, intakeexceptions,data-qualitytrendsandreviewqueue. Evidence,ruleversion,reviewerdecision,reason/commentandcompletehistory. |
| Verificationresult | Outcome,fieldsdisclosed,datasource,verificationtimestampandlimitations. |
| Auditview | Searchableactionhistorywithrole-appropriatemaskingandexportrestrictions. |

- Useplain-languagevalidationmessages,notonlytechnicalcodes.

- Usecolorplustext/iconlabels;neverrelyoncoloralonetoindicaterisk.

- Showdatafreshnessandsourcecontextforallmetrics.

- AvoiddisplayingfullAPAARIDsinlists;maskthemunlesstheroleandtaskrequirethefullvalue.

- Useserver-sidepagination/filteringforlargelists.

- Makeempty,loading,permission-deniedanderrorstatesexplicit.

## 12.Security,privacyandgovernancerequirements

Theplatformprocessessensitiveeducationalandidentity-linkedinformation.Securityandprivacycontrolsmustbebuiltinto thedesign.ThisPRDdoesnotdeterminethefinal legalbasis,consentwording,data-controllerresponsibilities,retentionperiod orproductioncomplianceobligations;thesemustbereviewedwiththeauthorizedinstitutionandapplicablelegal/security stakeholders.

| Control area Transportandstorage Authentication Authorization Secrets Inputsecurity files. Audit Privacy | Requirement TLSintransit;encryptionatrestfordatabase/backupsusingmanagedkeyswhereavailable. PreferestablishedOIDC/SSO. Ifprototypepasswordsareused,storeonlyamodernsaltedpasswordhash; enforcesafesession/tokenexpiry. Leastprivilege,institution-scopedaccess,denybydefault,server-sidechecksoneverysensitiveroute. Keepdatabasecredentials,signingkeysandexternalAPIcredentialsoutofsourcecontrolandlogs. Validatefiles,rowcounts,encodings,stringsandqueryfilters;protectagainstCSVformulainjectioninexported Recordactor,action, target, timestamp,outcomeandcorrelationID;protectlogsfromunauthorizedalteration. Collectminimumnecessaryfields,maskidentifiers, limitverificationdisclosureandavoidunnecessaryraw-file retention. |
| --- | --- |
| Abuseprevention | Rate-limit,restrictbulksearch/export,detectrepeatedenumerationandmonitorsuspiciousaccess. |
| Backupsandrecovery | Encryptedbackups, testedrestoreprocedureanddocumentedrecoveryobjectivesbeforeproduction. |
|   | Patchdependencies,runvulnerability/dependencyscans,useseparateenvironmentsandleast-privilege |
| Secureoperations | databaseaccounts. |
| Incidentresponse | Definereporting,containment,evidencepreservation,communicationandrecoverystepsbeforeproductionuse. |
| Datalifecycle | Documentpurpose,access,correction,retention,archivalanddeletionrules;getpolicyapproval. |

## 12.1Specificprivacydecisions

- DonotputAPAARIDorpersonal informationinaQRcodeURLorpublicURLparameters.

- Useanopaquerandomverificationtokenwithexpiry/revocationifQRverificationisimplemented.


- Averifiershouldreceiveonlyaminimalresultrequiredforthedeclaredpurpose.

- Donotlogrequestbodiescontainingstudentdataoraccesstokens.

- Treatsynthetictestdataassyntheticandavoidgeneratingexamplesthataccidentallyresemblerealpersonalrecords.

- DecidewhetherthecentralrepositorystoresrawAPAARID,aprotectedlookuprepresentation,orbothonlyafter reviewingoperational lookupneedsandapprovedsecuritydesign.Hashingaloneisnotautomaticallysufficientifidentifiers haveapredictableformat.

- Consentisnotablanketpermissionforallprocessing; implementtheapplicableauthorizedpurposeandnoticesapproved bytheresponsibleauthority.

## 13.Non-functionalrequirements

| ID Qualityattribute | Requirement/proposed target |
| --- | --- |
| NFR-01 Reliability | Afailedbatchmustnotleaveapartial,ambiguouscommit.Useexplicitbatchstatesandtransactions. |
| NFR-02 Performance | Prototype: target1,000syntheticCSVrowsunder10secondsintheagreeddemoenvironment; |
|   | measureratherthanassume. |
| NFR-03 Scalability | Usepagination,databaseindexes,boundeduploadsandbackgroundprocessingifbatchesgrow |
|   | beyondsynchronouslimits. |
| NFR-04 Availability | Prototypeavailabilityisbest-effort.Productionuptimetargetmustbesetbytheserviceowner. |
| NFR-05 Maintainability | Modularmonolith,documenteddomainrules,migrationscriptsandautomatedtests. |
| NFR-06 Interoperability NFR-07 | VersionedRESTAPI,stableidentifiers,documentedschemaandintegrationadapterboundary. |
| Accessibility NFR-08 Observability NFR-09 Portability NFR-10 Dataintegrity | Keyboard-operableforms, labels,sufficientcontrastandunderstandablestatusmessages. Structuredtracing,requestIDs,healthchecksandnon-sensitiveoperationalmetrics. Containerizedserviceanddocumentedenvironmentconfiguration. Transactionsandconstraintsprotectcriticaloperations;allcorrectionsareattributable. |

## 14.Reportinganddashboarddefinitions

| Metric |   | Definition/caution |
| --- | --- | --- |
| Submittedrecords |   | Numberofrecordsinbatchesreceivedduringaselectedperiod;distinguishsubmittedfromaccepted. |
| Acceptedrecords |   | Recordsthatpassedhardvalidationandwerecommittedaccordingtotheconfiguredworkflow. |
| Rejectedrecords Warnings/reviewcases |   | Recordsblockedfromcommitduetoharderrors. Recordsacceptedonlyifpolicypermits,orheldforhumanreview;definestatusesprecisely. |
| Actualenrolment Intakeutilization |   | Countofactiveacceptedadmissionsunder theconfiguredrule,notmerelycountofstudentidentities. Applicableactualcountdividedbytherelevantapprovedcapacity;handlezero/unconfiguredcapacityexplicitly. |
| Intakeexceptions |   | Caseswherethecalculatedcountexceedsaconfiguredrule,withsourceandruleversion. |
| Duplicatecandidates |   | Potentialconflictsrequiringreview;donotequateeveryduplicateflagwithfraud. |
| Dataqualitytrend Verificationactivity | charts. | Rateofissuesbycategoryovertime;accountforchangesinschema/rules. Requests,outcomes,responsetimesandaccess-deniedevents;avoidunnecessarypersonaldatainaggregate |

## 15.Testingandqualityassurance

## 15.1Requiredtestlayers

- Unittests:APAARfieldchecks,date/statusrules,duplicatelogic, intakecalculationsanderrormapping.


- Integrationtests:databaseconstraints,batchtransactions,role/scopeenforcementandauditeventcreation.

- APItests:valid/invalidpayloads,pagination,authentication,authorizationandrate-limitbehavior.

- Frontendtests:uploadstates,row-levelerrordisplay,role-dependentnavigationandpermissionerrors.

- Securitytests:cross-institutionaccess,IDORattempts,oversizedfiles,malformedCSV,injectionpayloads, tokenexpiry andexportlimits.

- End-to-enddemotest:uploadaknowndataset,verifyexpectedflags,resolveoneissueandconfirmdashboardcounts update.

## 15.2Seeddatasetscenarios

| Case | Expectedresult |   |
| --- | --- | --- |
| Validadmissionwithinconfiguredcapacity |   | Accepted;countedinthecorrectinstitution/programme/year. Rejectedorheldaccordingtoconfiguredhard-validationpolicy;clearreason |
| MissingAPAARID | shown. | Flaggedasformat-invalidonlyiftheformatspecificationisconfirmed;never |
| MalformedAPAARID DuplicateAPAARIDinsamebatch |   | claimauthoritativeverification. Duplicatecandidateidentifiedwithrowreferences. |
| Historicalwithdrawnadmission | SameIDwithconflictingactiveadmissionsacrossinstitutions Cross-institutionreviewissue;noautomaticfraudverdict. | Notcountedasactiveiftheconfiguredstatusrulesaysso;historyretained. |
| Admissionsexceedregularintake |   | Intakeexceptionwithcalculatedcountandconfiguredcapacity. Countedaccordingtotheexplicitlyconfigureddemo/policyrule,notassumedto |
| Supernumeraryseats | beregularseats. |   |
| Batchfailshalfwaythroughcommit Verifierasksforexcessfields | Unauthorizedinstitutionuserqueriesanotherinstitution unambiguous. | Deniedandloggedwithoutrevealingrestrictedrecorddetails. Transaction/commitpolicypreventssilentpartialsuccess;batchstatusis Fieldsarefiltered/deniedandthedisclosureattemptisauditable. |

## 16.Deliveryplanfora2– 7dayteam

| Timebox Deliverable Requirementslock,schema,APIcontract,seed | Definitionofdone AgreedCSVcolumns,ruledefinitions,repositorysetupandone |
| --- | --- |
| Day1/first20% data | runninghealthendpoint. |
| RustAPI,databasemigrations, ingestionand | Uploadreturnsrow-levelresults;corerulescoveredbyunit |
| Day1– 2/next30% validation | tests. |
|   | Institutioncanuploadandviewresults;regulatordashboard |
| Day2– 3/next25% ReactUIanddashboards | reflectsactualseededdata. Restrictedaccessworks;oneissuecanbereviewedwith |
| Day3– 4/next15% Reviewworkflow,rolechecksandauditevents | traceablehistory. Cleandemorun,knownlimitationsdocumented,nocritical |
| Remainingtime Testing,deployment,demo,documentation | access-controlbugs. |

## 16.1If thereisonlyonedeveloper

- UseCSVratherthansupportingCSVandXLSXimmediately.

- UseasingleRustAPIserviceandPostgreSQL;donotbuildmicroservices.

- Startwithdeterministicrules,notanMLmodel.

- Buildonlytworolesinitially:institutionuserandAICTEadministrator;representverifieraccessasacontrolleddemo extensioniftimeallows.

- Maketheend-to-endhappypathworkbeforevisualpolish.


## 16.2Iftherearetwodevelopers

- DeveloperA:RustAPI,schema/migrations, ingestion,validationandtestfixtures.

- DeveloperB:ReactUI,uploadexperience,dashboardandAPIintegration.

- Both:agreeonJSONschemasanderrorcodesearly;jointlytestauthorizationandthefinaldemo.

## 17.Risks,assumptionsandmitigations

| Risk/assumption NoauthorizedliveAPAARAPI access Intakerulesdifferby category/year DuplicateIDsaretreatedasfraud Overcollectionofpersonaldata Unauthorizedcross-institution access CSV/XLSXparsingedgecases | Impact Cannotclaimauthoritativeidentity verification. Falsepositivesorincorrect regulatoryconclusions. Legitimatetransfers/historical recordsmaybewronglyflagged. Privacyandbreachimpact increases. Highconfidentialityrisk. Badrecordsorcrashes. Coreworkflowremains | Mitigation/decisionneeded Useamockadapterclearlylabelledsimulated;confirmofficial integrationroutewiththerelevantauthority. Makerulesconfigurable/versioned;obtainauthoritativepolicy interpretation. Usereviewstatesandadmissionlifecycle;explainevidenceand permitcorrection. Dataminimizationandrole-specificfielddisclosure. Server-sidetenantscoping,negativetestsandauditmonitoring. Filelimits,strictschema,safeparsing,row-levelerror reportingand fuzz/edgetests. Useestablishedcrates,minimalmodules,andavoidadvanced |
| --- | --- | --- |
| Rustunfamiliarityslowsdelivery | incomplete. Lowjudgeconfidenceandriskof | async/distributedpatterns. Rule-basedindicatorsfirst;showevidenceandkeepahuman |
| AIscoreisnotexplainable Prototypemistakenfor | unfairflags. Overstatedsecurityor integration | reviewerincontrol. Documentdemolimitations,mockservices,securitygapsand |
| production-ready Unclearownership/retention | claims. Datagovernancecannotbe | productiongates. Resolvecontroller/processorresponsibilitiesandapprovedlifecycle |
| policy | finalized. | policybeforerealdata. |

## 18.Dependenciesanddecisionstoresolve

Thefollowingquestionsmustbeansweredbeforeproductionimplementation.Forthehackathon,explicitlymarkunresolved itemsasassumptionsratherthansilentlydecidingthem.

- WhatistheauthoritativesourceandexactspecificationforAPAARIDformatandliveverification?

- IsthereanauthorizedAPI,sandbox,credentialprocess,ratelimitandpermittedpurposeforverification?

- Whatistheofficialsourceofapprovedintakeandhowisitversionedbyinstitution/programme/academicyear?

- Whataretheapprovedcountingrulesforregular,supernumerary,NRI,OCIandforeign-nationalcategories?

- Whichstudentattributesaremandatory,optional,restrictedorprohibitedfromstorage?

- Whoisthedatacontroller/owner,andwhocanapproveaccess,correction,retentionanddeletionpolicies?

- Whatauthentication/SSOservicewill institutionsandAICTEuse?

- Whatlevelofassuranceisrequiredforemployersorotherexternalverifiers?

- Whatareexpecteddatavolume,peakuploadsize,concurrency,uptimeandrecoveryobjectives?

- Whatintegrationsanddataformatsexist inthecurrentAICTEandinstitutionalsystems?

- Whataretherequiredreportingformats,exportpermissionsandaudit retentionrules?

- Whathosting,procurement,securityreviewandaccessibilityrequirementsapplytoproduction?


## 19.AcceptancecriteriaforMVPsign-off

- Auserintheinstitutionrolecanauthenticateandcannotaccessanotherinstitution'srecords.

- AnauthorizedusercanuploadtheagreedsampleCSVandreceivebatchstatusplusrow-levelvalidationresults.

- MissingIDs,duplicatecandidates, inconsistentfieldsandconfiguredintakeviolationsaredetectedintheseeded scenarios.

- Acceptedcountsanddashboardvaluesarecomputedfrompersisteddata,nothardcoded.

- Admissionhistorysupportsinactive/historicalrecordswithoutcountingthemasactivewhenrulessayotherwise.

- Everycommittedbatchandreviewdecisionhasanattributableauditevent.

- AnAICTErolecanviewcross-institutionaggregatesandareviewqueue.

- Averifierflow,ifincluded,returnsonlyallowedfieldsandrecordstherequest;anymockverificationislabelledsimulated.

- APIerrorsarestableandsafe;sensitivevaluesandtokensarenotemittedinlogs.

- Testscovercorevalidationlogic,databasetransactionbehaviorandauthorizationfailures.

- READMEexplainshowtoruntheRustservice,migratethedatabase, loadseeddata,runtestsandstartthefrontend.

- KnownlimitationsandproductiondependenciesaredocumentedinthedemoandPRD.

- 20.Suggestedrepositorystructure

```
.
├── PRD.md
├── BUILD_PLAN.md
├── README.md
├── docker-compose.yml
├── .env.example
├── backend/
│   ├── Cargo.toml
│   ├── rust-toolchain.toml
│   ├── migrations/
│   ├── sample-data/
│   ├── src/
│   │   ├── main.rs
│   │   ├── config.rs
│   │   ├── state.rs
│   │   ├── error.rs
│   │   ├── routes/
│   │   │   ├── mod.rs
│   │   │   ├── auth.rs
│   │   │   ├── institutions.rs
│   │   │   ├── batches.rs
│   │   │   ├── admissions.rs
│   │   │   ├── issues.rs
│   │   │   ├── dashboards.rs
│   │   │   ├── verifications.rs
│   │   │   └── reports.rs
│   │   ├── domain/
│   │   │   ├── mod.rs
│   │   │   ├── admission.rs
│   │   │   ├── intake.rs
│   │   │   ├── validation.rs
│   │   │   └── verification.rs
│   │   ├── services/
│   │   │   ├── mod.rs
│   │   │   ├── ingestion.rs
│   │   │   ├── validation.rs
│   │   │   ├── duplicate_detection.rs
│   │   │   ├── intake_monitoring.rs
│   │   │   └── audit.rs
│   │   ├── repositories/
│   │   │   ├── mod.rs
│   │   │   ├── institutions.rs
│   │   │   ├── students.rs
│   │   │   ├── admissions.rs
│   │   │   ├── batches.rs
│   │   │   └── issues.rs
│   │   ├── auth/
│   │   │   ├── mod.rs
│   │   │   └── policy.rs
│   │   └── integrations/
│   │       ├── mod.rs
│   │       └── academic_identity.rs
│   └── tests/
│       ├── api_authz.rs
│       └── api_batches.rs
├── frontend/
│   ├── package.json
│   ├── vite.config.ts
│   ├── tsconfig.json
│   ├── index.html
│   └── src/
│       ├── main.tsx
│       ├── App.tsx
│       ├── api/
│       ├── components/
│       ├── pages/
│       └── types/
└── docs/
    ├── openapi.yaml
    ├── data-dictionary.md
    ├── demo-script.md
    └── assumptions.md
```

## 21.Rust implementationnotes

- Representdomainstateswithenums(e.g.,BatchStatus,AdmissionStatus, IssueStatus)ratherthanarbitrarystrings.

- Keepdomainvalidationfunctionspurewherepossiblesotheycanbeunit-testedwithoutadatabase.

- Usetypedrequest/responsestructsandvalidateattheAPIboundary;donottrustdeserializedinputmerelybecauseitfitsa struct.

- KeepSQLtransactionsshortanddefineconflictbehaviorexplicitly.

- Useadatabaseuniquenessstrategythatmatchesbusinesssemantics.AuniqueindexonAPAARIDalonemayincorrectly prohibitlegitimatehistoricalormulti-context records.

- AvoidblockingCPU-heavyfileparsingontheasyncruntime;keepfilesizeboundedandconsiderablockingtaskor backgroundjobifnecessary.

- Usegracefulshutdown,connectionpooling, timeoutsandrequestbodylimits.

- Pindependenciesandruncargofmt,cargoclippyandcargotest inthebuildpipeline.

- Rundependencyvulnerabilitychecksandsecretscanningbeforesharingademodeployment.

- DonotintroduceanAImodeluntilrules, labels,dataqualityandevaluationcriteriaaredefined.

## 22.Demonarrativeforjudges

Recommended3– 5minutedemonstration:

- 18. Showtheinstitutiondashboardandapprovedintakeconfiguration.

- 19. UploadasyntheticCSVcontainingavalidrecord,missingAPAARID,duplicatecandidateandintakeoverrun.

- 20. Showthevalidationsummaryandopenonerecordtoexplaintheexactruleandevidence.

- 21. SwitchtotheAICTEroleandshowinstitution-levelmetricsandtheexceptionqueue.

- 22. Resolveordismissoneissuewithareason;showtheaudittrailandupdateddashboard.

- 23. Demonstratelimitedverification,ifbuilt,andclearlystatewhetheritusesamockorauthorizedlivesource.

- 24. Closewiththeimpact:consistentrecords,explainableexceptions,privacy-awareaccessandapathtointegration.

Avoidsaying'AIverifiesAPAAR'ifthesystemonlycheckssyntax.Usepreciselanguagesuchas'formatvalidation', 'duplicate candidatedetection', 'intakerulecheck',and'authorizedexternalverificationadapter'.


## 23.Futureroadmap

| Phase | Scope | Entrycriteria |
| --- | --- | --- |
|   | CSVupload,deterministicvalidation,duplicateflags, intakedashboard, | AllMVPacceptancecriteriapasson |
| Prototype |   |   |
|   | roles,audittrail. | syntheticdata. |
|   | Approvedidentityintegration,institutionalAPIadapter,correction | Writtenapprovalfordataaccessand |
| Pilot |   |   |
|   | workflow,monitoreddeployment,policy-reviewedprivacycontrols. | confirmedbusinessrules. |
|   | SSO,operationalmonitoring,backup/restoretests,loadtesting, | Serviceowner,supportmodel,risk |
| Productionhardening | incidentresponse,securityassessment,accessibilityandretention | assessmentandservice-leveltargets |
|   | controls. | agreed. |
|   | Backgroundbatchprocessing,advancedanalytics,additional | Reliabledata,measurablebaselineand |
| Scaleandanalytics |   |   |
|   | integrationsandcarefullyevaluatedanomalyscoring. | governanceformodel/rulechanges. |

## 24.Glossary

| Term APAARID Approvedintake Actualenrolment Supernumeraryseat | MeaninginthisPRD Academicidentityidentifier referencedbytheproblemstatement;formatandauthoritativeverificationrules mustcomefromapprovedsources. Authorizedcapacityforadefinedinstitution/programme/academicyear,subjecttotheapplicableofficialrules. Countderivedfromadmissionsthatqualifyundertheconfiguredstatusandcountingrules. Aseatcategoryrequiringseparatepolicy-awaretreatmentintheintakecalculation. |
| --- | --- |
| Validationissue | Adetectedruleviolation,inconsistencyorreviewflag;notnecessarilyevidenceofmisconduct. |
| Authoritativeverification | Acheckagainstanapprovedsourceusinganauthorizedinterface/process,notmerelyalocalformatcheck. |
| RBAC | Role-basedaccesscontrol;mustbecombinedwithrecord/institutionscopingwhereneeded. |
| MVP | Minimumviableproduct: thesmallestcompleteworkflowthatdemonstratescorevalue. |
| Modularmonolith | Onedeployableservicedividedintowell-definedinternalmodules,suitableforasmallteam. |

## 25.Final recommendation

BuildaRustmodularmonolithwithAxum,Tokio,SQLxandPostgreSQL;useReact forthefrontend. Implementsecure institutionalscoping,CSVingestion,deterministicvalidation,cross-institutionduplicatecandidatedetection,policy-configured intakemonitoring,anAICTEdashboardandanaudit trailbeforeaddingadvancedAI.

Fora2– 7daybuild,themostvaluableproofisacomplete, testedworkflowonsyntheticdata.TreatliveAPAARverification, official intakerules,real institutiondataandproductionsecurityassuranceasexplicitintegration/governancegates.

ThisPRDshouldbereviewedwiththerelevantAICTE/APAARstakeholdersbeforeit isusedasaproductionspecification.The suppliedproblemstatementestablishestheproductdirection,butitdoesnotprovidetheexactAPIcontracts, legalbasis, intake formulas,data-retentionschedule,service-levelobjectivesorproductionsecuritystandards.
