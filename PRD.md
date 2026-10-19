PRODUCTREQUIREMENTSDOCUMENT
APAAR-BasedStudentData &VerificationPlatform Management
Supabase-firstfull-stackimplementationforAICTE-approvedinstitutions
Documentfield	Value 26263
ProblemStatementID	
Organization/Department	AICTE/RegulationBureau
Category/Theme	Software/SmartAutomation
Documenttype	ProductRequirementsDocument(PRD)
Version/status	1.0—Proposedspecificationforplanning
Date	19October2026
Intendedteam	1– 2developers;2– 7dayprototypewindow


IMPORTANTSCOPENOTE
ThisPRDisaproposedproductdesignderivedfromthesuppliedproblemstatement. It isnotanofficial AICTEspecification, legalopinion,orconfirmationofaccesstoliveAPAAR/governmentAPIs.Those

interfaces,rules,andapprovalsmustbevalidatedwithauthorizedstakeholders.
Documentcontrol andhowtousethisPRD
Thisdocumentdefinestheproblem,users,requirements,workflows,datamodel,technologyarchitecture,securityexpectations, deliveryplan,acceptancecriteria,risksandopendecisionsforaprototypethatcanlaterevolvetowardaproductionsystem.
- UsetheP0requirementsasthesmallestcoherenthackathonMVP.
- TreatP1itemsasvaluableextensionsaftertheprimaryworkflowisstable.
- Treatallexternal identityverification,policy-specificintakecalculations,andregulatoryintegrationsasgateduntilofficially confirmed.
- Donotuserealstudentpersonaldatainthehackathondemo.Usesyntheticrecords.
Priority	Meaning
P0—Musthave	Requiredtodemonstratethecoreworkflowandthemostimportantproblem-statementrequirements.
P1—Shouldhave	High-valuecapabilitythatimprovestrust,usabilityorevaluatorconfidence.
P2—Couldhave	Optionalextension;donotjeopardizethecoreMVPtobuildit.


1.Executivesummary
AICTEreceivesstudent-relatedinformationthroughdifferentsubmissionmechanisms.Thesuppliedproblemstatement explainsthatPGDMinstitutionsprovidedetailedstudentdataannually,whileotherAICTE-approvedinstitutionsprimarily submitaggregateenrolmentinformationduringapproval.Thismakescomplete,authentic,comparableandverifiable student-levelrecordsdifficulttomaintaincentrally.

Theproposedplatformprovidesastandardizedpathforinstitutionstosubmitandupdatestudent-levelrecordslinkedto APAARID,validatetherecords,detectduplicatesandinconsistencies,compareactualenrolmentwithapprovedintake,and makeauthorizedinformationavailablethroughdashboards,reportsandverificationworkflows.

ThesystemshouldbedesignedasanAPI-firstregulatorydataplatform.Forthe2–7dayprototype,useReact+TypeScript+ViteforthefrontendandSupabaseasthemanagedbackendplatform.SupabaseprovidesPostgreSQL,authentication,storageandEdgeFunctions.PostgreSQLconstraints,RowLevelSecurity(RLS),server-sideauthorization,testingandgovernanceareessentialforsecurityandregulatoryreadiness.
Productvision
Createareliable,privacy-aware,interoperablestudent-recordecosystemthathelpsauthorizedstakeholdersunderstandwhat wassubmitted,whatwasvalidated,what remainsunresolved,andwhichrecordscanbeverifiedforadefinedpurpose.
Productoutcome
Anauthorizedinstitutioncanuploadabatch,receiverecord-levelvalidationresults,resolveissues,andsubmitaccepted records.AnAICTEadministratorcanmonitorenrolmentandexceptiontrendsacrossinstitutions.Anauthorizedverifiercan checkalimitedrecordstatuswithoutreceivingunrestrictedaccesstothecentralrepository.
2.Problemstatement,needandeffects
2.1Currentproblem
- Submissionpracticesdifferbyinstitutiontypeandprocess.
- Aggregateenrolmenttotalsdonotprovidethestudent-leveldetailneededforindividualrecordchecks.
- Missing,malformed,duplicatedorinconsistentidentifierscanunderminethequalityofacentralrepository.
- Admissionsmayneedtobecomparedwithapprovedintakeatthecorrectinstitution/programme/academic-yearlevel.
- Authorizedstakeholdersneedatrustworthywaytoverifypermittedinformationwithoutexposingallstudentdetails.
- Regulatoryreportingismoreusefulwhendatadefinitionsandvalidationrulesareconsistentacrossinstitutions.
2.2Whoandwhat isaffected?
Stakeholder	Effect of the current gap	Desired outcome
AICTE/Regulation Bureau	Limited consistent visibility into student-level records and intake compliance.	Central monitoring, exception queues, trustworthy reports and trend analysis.
Institution administrators	Repeated formatting, submission corrections and uncertainty about rejected records.	Clear upload templates, actionable errors, correction workflow and submission status.
Students	Potential identity mismatch, duplicate records or inaccurate institutional data.	Accurate records, correction route, limited disclosure and accountable access.
Authorized verifiers	Difficulty establishing whether a student record is valid and current.	Purpose-limited verification result with clear provenance and timestamp.
Degree-awarding bodies	Need to associate student IDs and APAAR IDs on certificates where required.	Defined field and display/integration support, subject to approved rules.
Policy and accreditation teams	Inconsistent data may limit comparison across programmes and institutions.	Comparable, traceable and appropriately aggregated reports.


2.3Whytheplatformisneeded
Theplatformshouldturndatasubmissionintoacontrolleddata-qualityandverificationprocess. Itsvalueisnotjuststoring records: itmustpreventavoidableerrors,makeunresolvedissuesvisible,supporthumanreviewofambiguouscases,and preserveatraceablehistoryofchanges.
2.4Expectedimpact
- Improvedcompletenessandconsistencyofstudent-levelrecords.
- Earlierdetectionofpossibleduplicateactiveenrolmentsandcapacityviolations.
- Reducedmanualeffort toidentifyerrorsinlargesubmissions.
- Fasteraccesstoauthorizedverificationoutcomes.
- Moreconsistentevidenceforregulatorymonitoringandpolicyanalysis.
Impactshouldbemeasuredusingoperationalmetricssuchasvalidationerrorrate,medianbatch-processingtime,percentage ofrecordswithresolvedissues,numberofintakeexceptionsreviewed,andverificationresponsetime.Theprototypeshould notclaimthatithasreducedfraudorimprovednationaldataqualitywithoutmeasuredevidence.
3.Productgoals,non-goalsandsuccessmetrics
3.1Goals
- Provideasecure,standardizedmechanismforinstitution-levelstudent-datauploadandupdates.
- UseAPAARIDastheproposedacademicidentitykeywhilepreservinginstitution-specificstudentIDsandadmission history.
- Validatemissing/invalididentifiers,duplicaterecords, inconsistentdataandintakeexceptions.
- Providerole-appropriateinstitutionandAICTEdashboards.
- Supportpurpose-limitedverificationforauthorizedstakeholders.
- Maintainauditablehistoryofsubmissions,corrections,reviewsandverificationrequests.
- ProvidereportandAPIcapabilitiessuitableforfutureintegration.
3.2Non-goalsforthe2– 7dayprototype
- ReplacingAPAAR,DigiLocker,AICTEorinstitutionalMIS/ERPsystems.
- Claiminglivegovernmentverificationwithoutanapprovedintegrationandcredentials.
- Buildinganationwideproductiondeploymentorcompletingaformalsecuritycertification.
- Makingautomatedfinaldeterminationsof fraud,eligibility,admissionlegalityorstudentmisconduct.
- UsingAI/MLasasubstituteforexplicitregulatoryrulesorhumanreview.
- Collectingunnecessarypersonal information, identitydocumentsorbiometricdata.
3.3Successmetrics
Metric	Prototype target (proposed)	Measurement
Duplicate detection	Detect exact APAAR-ID collisions in the seeded dataset.	Automated test suite with known valid/invalid sample records.
Intake check	Test same-upload and cross-institution cases.	Compare calculated counts to known fixtures.
Batch validation correctness	All seeded test cases produce expected results.	Correctly flag configured cases exceeding applicable demo intake.
Traceability	Every create/update/review action creates an audit event.	Integration test and audit-log inspection.
Access control	Unauthorized role cannot access restricted endpoints.	Negative authorization tests.
Usability	User can upload a batch and understand failed rows without developer help.	Observed task walkthrough.
Performance	Process 1,000 synthetic rows in under 10 seconds on the chosen demo environment.	Repeatable local benchmark; not a production SLA.


Thesetargetsareinitialengineeringgoalsfortheprototype,notofficialAICTEservice-levelrequirements.Revisitthemafter defininghostingresources,datavolumeandoperatingconstraints.
4.Users,rolesandpermissions
Role	Primary responsibilities	Default access boundaries
Institution Admin	Manage institution profile and approved programmes; upload/update own institution records; review validation results.	Only institution-scoped records; cannot administer other institutions.
Institution Data Operator	Prepare batches, submit records and fix validation issues.	Institution-scoped access; no global policy or user administration.
AICTE Regulator / Admin	Monitor institutions, review exceptions, configure authorized rules, produce regulatory reports.	Access limited by assigned regulatory role and audited purpose.
Authorized Verifier	Request a defined verification result for an identified record.	Only minimum approved fields; no general student search or bulk export by default.
Auditor / Read-only Reviewer	Inspect relevant submission and access history.	Read-only access to assigned audit scope.
Student (future phase)	View permitted record status and request correction through a defined workflow.	Only own record after a reliable identity/authorization method exists.
System Operator	Manage deployment, service health and backups.	Operational access should not automatically grant unrestricted business-data access.


Authorizationmustbeenforcedinthebackendoneveryrelevantrequest.HidingabuttoninReactisnotanaccess-control mechanism. Institutionscopemustcomefromtheauthenticateduser'strustedserver-sideidentity,notfromanuntrusted institution_idsuppliedbytheclient.
5.Coreuserjourneys
5.1Institutionuploadsastudentbatch
-
  1. Usersignsinandselectstheinstitution/programme/academicyeartheyareauthorizedtomanage.
-
  2. UserdownloadsorfollowstheversionedCSVtemplateanduploadsafile.
-
  3. APIchecksfiletype,size,encoding,requiredcolumnsandrowcountlimits.
-
  4. Systemcreatesabatchrecordandvalidateseachrowwithoutsilentlydiscardingerrors.
-
  5. UIshowscountsforacceptedcandidates,warnings,rejectedrowsandduplicatecandidates.
-
  6. Userdownloadsarow-levelerrorreport,correctsthesourcefileandresubmitsorusesanauthorizedcorrectionworkflow.
-
  7. Acceptedrecordsarecommittedtransactionallywithanauditeventandabatchsummary.
5.2AICTEreviewsanintakeexception
-
  8. Dashboarddisplaysapprovedintakeandactualadmissionsusingtheconfiguredcountingrule.
-
  9. Anexceptioniscreatedwhentheapplicablelimitisexceededorthedataisinconsistent.
-
  10. Revieweropenstheexceptionandseesthecalculation,sourcerecordsandruleversion.
-
  11. Reviewerrecordsanoutcomesuchasconfirmedissue,datacorrectionrequested,ordismissedwithreason.
-
  12. Thesystempreservestheoriginalalertandreviewhistory; itdoesnoterasetheevidencewhenstatuschanges.
5.3Authorizedstakeholderverifiesarecord
-
  13. Verifierauthenticatesandselectsanapprovedverificationpurpose.
-
  14. Thesystemcheckstheverifier'srole,scope,purposeandpermitteddatafields.
-
  15. Thesystemreturnsalimitedresultsuchasverifiedagainsttheplatform'srecords,needsreview,ornotfound.
-
  16. Theresponsestatesthesourceandtimestamp;itdoesnotimplyliveAPAARauthorityunlessthatsourcewasactually consulted.
-
  17. Therequestanddisclosedfieldsarerecordedintheaudittrail.
5.4Studentrecordcorrection
Acorrectionshouldcreateanewversionoratraceablechangeeventratherthanoverwritingtheonlyhistoricalcopy. Dependingonthefieldandpolicy,thecorrectionmayrequireinstitutionalapprovalorregulatorreview.Thesystemshould preservewhorequested,approvedandappliedthechange.
6.Functional requirements
ID	Requirement	Priority	Acceptance criteria
FR-01	Authentication: users can sign in and obtain a server-validated session/token.	P0	Invalid credentials fail; protected endpoints reject unauthenticated calls.
FR-02	Role-based access and institution scoping.	P0	Cross-institution access attempts are rejected and logged.
FR-03	Institution and programme configuration.	P0	Authorized admin can configure institution, programme, academic year and intake fixture.
FR-04	Bulk upload through a documented CSV template.	P0	Valid file creates a batch; malformed file returns actionable errors.
FR-05	Row-level validation report.	P0	Each row receives accepted/rejected/warning status and one or more reason codes.
FR-06	APAAR field validation.	P0	Missing/format-invalid values are flagged. Format checks are explicitly not identity verification.
FR-07	Duplicate detection within a batch and across institution records.	P0	Exact collisions are detected; ambiguous cases are flagged for review.
FR-08	Admission and intake comparison.	P0	Counts are calculated by institution/programme/year and applicable seat rules.
FR-09	Central student and admission registry.	P0	Accepted records can be searched by authorized users and retain admission history.
FR-10	Record correction / review workflow.	P0	Issue can be assigned, commented on and resolved without deleting history.
FR-11	Institution dashboard.	P0	Shows own accepted records, issues, batch status and configured intake utilization.
FR-12	AICTE dashboard.	P0	Shows aggregate institution/programme metrics and prioritized exceptions.
FR-13	Audit trail.	P0	Upload, edit, review, role change and verification actions produce attributable events.
FR-14	Purpose-limited verification endpoint.	P1	Verifier receives only allowed fields; request is authenticated and audited.
FR-15	Report export.	P1	Authorized user exports filtered aggregates or row-level issues within permitted scope.
FR-16	REST integration API and versioning.	P1	API contract is documented and uses stable error codes and schema versions.
FR-17	QR-based verification link.	P2	QR resolves to a protected or deliberately minimal verification page; no secret data is embedded.
FR-18	Explainable anomaly score.	P2	Score is derived from documented rules and shows the evidence; never makes an automatic fraud verdict.
FR-19	Certificate display support for APAAR ID plus student ID.	P2	Field mapping and output format are demonstrable; official issuance integration remains separate.


7.Datavalidationandregulatoryrules
7.1Validationclasses
Class Structural	Examples Missingrequiredcolumn, invaliddate,unsupportedfile,		Systembehavior Rejectfileor rowwithaclearerrorcode.
Field-level	oversizedupload. MissingAPAARID,emptystudentID,malformed		Rejectormarkasneedingcorrectionaccordingtoconfigured
Uniqueness	programmecode. SameAPAARIDrepeatedinabatchorlinkedtomultiple		rule. Detectexactduplicate;cross-institutionconflictbecomesa
Referential	activeadmissions. Programmeorinstitutiondoesnotexist;academicyear		reviewableflag. Rejectorquarantineuntilmasterdataiscorrected.
Capacity	notconfigured. Countexceedsapprovedintakefora		Createanintakeexceptionwithcalculationandpolicy-rule
Consistency	programme/year/seatcategory. Admissiondateoutsideallowedperiod;categorytotals		version. Flagwiththespecificruleandrelevantvalues.
Anomaly	donotreconcile. Unusualchangefrompriorsubmissionorunusuallyhigh		Generateexplainableriskindicatorforhumanreview.
	correctionrate.		


7.2Intakecountingmustbepolicy-configurable
Donothard-codeasingleuniversal formulaforeveryseatcategory.Theapplicabletreatmentofsupernumeraryseats,NRI category,OCIstatusandforeign-nationalstatusmustbeestablishedfromofficialAICTErulesandtherelevantacademic-year approval.Storeapprovedintakeandcountingruleswithaneffectivedate/version.

Forademo,defineanexplicitsamplerule:countacceptedactiveadmissionsbyinstitution+programme+academicyear, then comparewiththeconfiguredregularintakeandseparatelyconfiguredapprovedsupernumerarycapacity.Label thisasademo ruleuntilvalidated.
7.3APAARvalidationlevels
- Level1:required/presentcheck.
- Level2:formatandlengthchecksonlyifanofficial formatspecificationhasbeenconfirmed.
- Level3:authoritativeliveverificationonlythroughanauthorizedAPI/processwithapprovedcredentials.
TheUIandAPImustdistinguishtheselevels.Avaluepassingformatchecksmustneverbelabelled'APAARverified'.
7.4Duplicatelogicandexceptions
- SameAPAARIDrepeatedinthesamebatch: flagasaduplicatecandidateandidentifyrownumbers.
- SameAPAARIDlinkedtomorethanoneactiveadmission:raiseacross-institutionconflictforreview.
- Historical,withdrawn,completedortransferredadmissionsmustberepresentedwithstatusanddatessolegitimatecases arenotautomaticallytreatedasfraud.
- Donotexposeastudent'sinstitutionhistorytoadifferentinstitutionunlessthatdisclosureisexplicitlyauthorized.
- Keepthedecision,evidenceandrevieweridentityfordismissedorconfirmedflags.
8.Datamodelproposal
Entity Institution	Importantfields id,official_code,name,status,created_at	Notes Official identifiersshouldbevalidatedagainst approvedmasterdata.
Programme	id, institution_id,code,name, level,status id, institution_id,programme_id,academic_year,regular_intake,	Institution-scopedprogrammecatalogue. Capacitymustbetraceabletoanapproval
ApprovedIntake	supernumerary_rules,source_reference,effective_version	sourceandeffectiveperiod.
StudentIdentity	id,apaar_id, institution_student_id(wherescoped),minimal	Avoidstoringunnecessarypersonaldata; identityuniquenessstrategyrequirespolicy
	permittedattributes,created_at	review.
Admission SubmissionBatch ValidationIssue VerificationRequest AuditEvent User/Role	id,student_identity_id, institution_id,programme_id, academic_year,admission_date,status,seat_category, source_batch_id id, institution_id,submitted_by,file_hash,schema_version,status, counts,submitted_at id,batch_id,row_reference,entity_id,rule_code,severity,details, status,resolved_by,resolved_at id,requester_id,purpose,record_ref, fields_disclosed,outcome, requested_at id,actor_id,action,entity_type,entity_id,timestamp,correlation_id, safe_change_summary id, login_identifier,role,institution_scope,status,auth_metadata	Multiplehistoricaladmissionsmaybevalid. Storefilemetadata;avoidretainingrawfiles longerthannecessary. Detailsmustavoidunnecessarypersonaldata. Auditaccessanddatadisclosure. Append-onlyapplication-leveleventhistory; restrictandmonitoraccess. Neverstoreplaintextpasswords.


8.1Databaseintegrity
- Useprimarykeysandforeignkeyswhereverthechosenrelationaldesignsupportsthem.
- Usedatabaseconstraintsforinvariantrulesthatmustholdunderconcurrentrequests.
- DefinehowanAPAARIDisnormalized,protectedandindexed;donotcasuallylogit.
- Usetransactionsforbatchcommitsandreview-statechanges.
- Useidempotencykeysorbatchfingerprintstopreventaccidentalduplicatesubmissions.
- StoretimestampsinUTCandrenderlocal timeintheUI.
- Defineretentionanddeletionpolicieswiththedatacontroller/authorizedinstitution;donotinventauniversalretention period.
9. Recommended technology stack and architecture
9.1 Recommended stack
Concern	Recommendation	Reason/caveat
Frontend	React+TypeScript+Vite	Fast development server and build workflow; good fit for a short hackathon prototype.
UI components	Tailwind CSS+shadcn/ui (or a simpler component library)	Fast, consistent dashboards and forms; keep setup minimal.
Data fetching	TanStack Query or a small typed service layer	Handles loading, caching and mutations without unnecessary state-management complexity.
Backend platform	Supabase	Managed platform combining PostgreSQL, Auth, Storage and Edge Functions.
Database	Supabase PostgreSQL	Relational constraints, indexes, transactions and migrations fit this registry and audit use case.
Authentication	Supabase Auth	Use managed authentication instead of implementing password storage and token issuance from scratch.
Authorization	PostgreSQL Row Level Security (RLS)+server-side checks	Enforce institution-scoped access using trusted identity claims; never rely only on hidden UI controls.
Server-side workflows	Supabase Edge Functions (TypeScript/Deno)	Use for privileged batch validation/commit, verification requests, reports and controlled integrations.
CSV ingestion	Optional frontend preview+server-side parsing/validation in an Edge Function	Browser preview improves usability, but authoritative validation must run server-side.
Schema migrations	Supabase CLI migrations	Version-control schema, indexes, RLS policies and seed data.
API	Supabase client for ordinary scoped CRUD; Edge Functions for privileged workflows	Keep response schemas and errors consistent. Never expose the service-role key to the browser.
Audit/observability	Append-only audit table+Edge Function logs	Record actor, action, target, timestamp and outcome without logging student identifiers or tokens.
Deployment	Vercel frontend+hosted Supabase project	Fast demo deployment; configure secrets and access controls before sharing.
Testing	Vitest/React Testing Library+RLS integration tests+Edge Function tests	Verify validation, cross-institution isolation, transaction behavior and audit events.


9.2 Logical modules
- frontend: React pages, components, upload wizard, dashboards and role-aware navigation.
- auth: Supabase Auth session handling and trusted identity claims.
- institutions: institution, programme, academic year and approved-intake configuration.
- ingestion: upload limits, CSV parsing, batch tracking and idempotency.
- validation: field, uniqueness, referential, intake and consistency rules.
- admissions: student identity and admission lifecycle operations.
- verification: purpose-limited lookup and field-disclosure policy.
- reporting: aggregate metrics, filtered reports and exports.
- audit: structured audit events and review history.
- database: PostgreSQL schema, constraints, indexes, RLS policies and migrations.
- edge-functions: privileged workflows and future external identity adapters.
9.3 Architecture pattern
For this team size, use a Supabase-first architecture instead of a separate microservice system. Use PostgreSQL for durable relational data and database-enforced constraints, RLS for row-level access boundaries, and Edge Functions for privileged orchestration. Keep business rules in small, testable functions and SQL routines where appropriate. Do not add a separate API server unless a validated requirement cannot be handled safely by Supabase.
9.4 External APAAR integration boundary
Define an academic identity verifier interface with a mock implementation for development and a separate production adapter only after the official endpoint, authentication, permitted fields, error model, rate limits and approval process are known. The mock must visibly identify itself as simulated. Do not fabricate a live API response. Keep external credentials in server-side secrets, never in frontend variables exposed to the browser.
9.5 Error handling
- Return consistent machine-readable error codes and safe human-readable messages from Edge Functions.
- Use suitable HTTP statuses: 400 invalid input, 401 unauthenticated, 403 unauthorized, 404 inaccessible/not found according to disclosure policy, 409 conflict, 413 upload too large, 422 row-level semantic errors when appropriate, and 500 unexpected failures.
- Never return SQL errors, stack traces, secrets or internal infrastructure details to clients.
- Distinguish file-level failure from row-level validation issues.
- Treat RLS denials as expected security outcomes and test them explicitly.
10.APIrequirements
SuggestedAPIsurface.SupabaseAuthandtheSupabaseclientcanhandleordinaryauthenticatedoperations;useEdgeFunctionsforprivilegedworkflows.Finalizefunctionnames,payloadsandresponseschemasbeforeimplementation.
Methodandroute	Purpose	Access
		Publicendpointwithratelimiting;
POST/api/v1/auth/login	PrototypeloginorOIDCinitiation	preferablydelegateauthtoan
GET/api/v1/me		identityprovider.
GET/api/v1/institutions	Returncurrentuser,roleandpermittedinstitutionscope Listinstitutionsavailabletocurrentrole	Authenticated Scoped/authenticated
GET/api/v1/institutions/{id}/intakes	Readapprovedintakeconfiguration	Scoped/authenticated
POST/api/v1/batches	CreatebatchanduploadCSV	Institutionoperator/admin
GET/api/v1/batches/{id}	Getbatchstatusandaggregateresults	Owner institutionorregulator
GET/api/v1/batches/{id}/issues	Getrow-levelvalidationissues	Owner institutionorregulator
POST/api/v1/batches/{id}/commit	Commiteligiblerecordsaftervalidation	Authorizedinstitutionuser;
GET/api/v1/admissions	Search/filterrecordswithinpermittedscope	idempotent Scoped/authenticated
GET/api/v1/dashboard/institution GET/api/v1/dashboard/aicte	Institutionmetrics Cross-institutionaggregatemetrics	Institutionrole AICTErole
POST/api/v1/issues/{id}/resolve	Resolveordismissanissuewithreason	Assignedreviewer/authorizedrole
POST/api/v1/verifications GET/api/v1/reports/enrolment GET/health/liveand/health/ready	Requestpurpose-limitedrecordverification Getorexportauthorizedenrolmentreport Processanddependencyhealthchecks	Authorizedverifier Scoped/authenticated Operationalendpoint;nosensitive details


10.1APIcontractrules
- Allprotectedoperationsrequireauthenticatedidentityandserver-sideauthorization.EnforceinstitutionscopewithRLSandrepeatchecksinsideEdgeFunctionsforprivilegedoperations.
- Validatequeryfiltersandpagination;cappagesizeandexportvolume.
- UseexplicitAPIversioningandstableerrorcodes.
- Setmaximumuploadsize,rowcount,processingtimeandrequestbodylimits.
- Userequest IDsforsupportandsafeauditcorrelation.
- Documentwhichfieldsmaycontainpersonaldataandwhicharereturnedtoeachrole.
- Rate-limitlogin,verificationandexpensiveexportendpoints.
- Useidempotencyordeduplicationsafeguardsforretriesandbatchcommits.
11.Frontendanduserexperiencerequirements
Screen Sign-in Institutiondashboard Uploadwizard	Requiredcontent Loginstatus,safeerrormessagesandsessionhandling. Latestbatchstatus,accepted/rejected/warningcounts,ownintakeutilizationandunresolvedissues. Templatedownload, fileconstraints,uploadprogress,validationsummaryandnextactions.
Batchdetail	Rownumber,safestudentreference, issuecode,explanation,severityanddownloadableerrorreport.
Student/admissiondetail	Onlypermittedattributes,currentstatus,historyandauthorizedcorrectionactions.
AICTEdashboard Issuereview	Institution/programmefilters,aggregateenrolment, intakeexceptions,data-qualitytrendsandreviewqueue. Evidence,ruleversion,reviewerdecision,reason/commentandcompletehistory.
Verificationresult	Outcome,fieldsdisclosed,datasource,verificationtimestampandlimitations.
Auditview	Searchableactionhistorywithrole-appropriatemaskingandexportrestrictions.


- Useplain-languagevalidationmessages,notonlytechnicalcodes.
- Usecolorplustext/iconlabels;neverrelyoncoloralonetoindicaterisk.
- Showdatafreshnessandsourcecontextforallmetrics.
- AvoiddisplayingfullAPAARIDsinlists;maskthemunlesstheroleandtaskrequirethefullvalue.
- Useserver-sidepagination/filteringforlargelists.
- Makeempty,loading,permission-deniedanderrorstatesexplicit.
12.Security,privacyandgovernancerequirements
Theplatformprocessessensitiveeducationalandidentity-linkedinformation.Securityandprivacycontrolsmustbebuiltinto thedesign.ThisPRDdoesnotdeterminethefinal legalbasis,consentwording,data-controllerresponsibilities,retentionperiod orproductioncomplianceobligations;thesemustbereviewedwiththeauthorizedinstitutionandapplicablelegal/security stakeholders.
Control area Transportandstorage Authentication Authorization Secrets Inputsecurity files. Audit Privacy	Requirement TLSintransit;encryptionatrestfordatabase/backupsusingmanagedkeyswhereavailable. PreferestablishedOIDC/SSO. Ifprototypepasswordsareused,storeonlyamodernsaltedpasswordhash; enforcesafesession/tokenexpiry. Leastprivilege,institution-scopedaccess,denybydefault,server-sidechecksoneverysensitiveroute. Keepdatabasecredentials,signingkeysandexternalAPIcredentialsoutofsourcecontrolandlogs. Validatefiles,rowcounts,encodings,stringsandqueryfilters;protectagainstCSVformulainjectioninexported Recordactor,action, target, timestamp,outcomeandcorrelationID;protectlogsfromunauthorizedalteration. Collectminimumnecessaryfields,maskidentifiers, limitverificationdisclosureandavoidunnecessaryraw-file retention.
Abuseprevention	Rate-limit,restrictbulksearch/export,detectrepeatedenumerationandmonitorsuspiciousaccess.
Backupsandrecovery	Encryptedbackups, testedrestoreprocedureanddocumentedrecoveryobjectivesbeforeproduction.
	Patchdependencies,runvulnerability/dependencyscans,useseparateenvironmentsandleast-privilege
Secureoperations	databaseaccounts.
Incidentresponse	Definereporting,containment,evidencepreservation,communicationandrecoverystepsbeforeproductionuse.
Datalifecycle	Documentpurpose,access,correction,retention,archivalanddeletionrules;getpolicyapproval.


12.1Specificprivacydecisions
- DonotputAPAARIDorpersonal informationinaQRcodeURLorpublicURLparameters.
- Useanopaquerandomverificationtokenwithexpiry/revocationifQRverificationisimplemented.
- Averifiershouldreceiveonlyaminimalresultrequiredforthedeclaredpurpose.
- Donotlogrequestbodiescontainingstudentdataoraccesstokens.
- Treatsynthetictestdataassyntheticandavoidgeneratingexamplesthataccidentallyresemblerealpersonalrecords.
- DecidewhetherthecentralrepositorystoresrawAPAARID,aprotectedlookuprepresentation,orbothonlyafter reviewingoperational lookupneedsandapprovedsecuritydesign.Hashingaloneisnotautomaticallysufficientifidentifiers haveapredictableformat.
- Consentisnotablanketpermissionforallprocessing; implementtheapplicableauthorizedpurposeandnoticesapproved bytheresponsibleauthority.
13.Non-functionalrequirements
ID Qualityattribute	Requirement/proposed target
NFR-01 Reliability	Afailedbatchmustnotleaveapartial,ambiguouscommit.Useexplicitbatchstatesandtransactions.
NFR-02 Performance	Prototype: target1,000syntheticCSVrowsunder10secondsintheagreeddemoenvironment;
	measureratherthanassume.
NFR-03 Scalability	Usepagination,databaseindexes,boundeduploadsandbackgroundprocessingifbatchesgrow
	beyondsynchronouslimits.
NFR-04 Availability	Prototypeavailabilityisbest-effort.Productionuptimetargetmustbesetbytheserviceowner.
NFR-05 Maintainability	Modularmonolith,documenteddomainrules,migrationscriptsandautomatedtests.
NFR-06 Interoperability NFR-07	VersionedRESTAPI,stableidentifiers,documentedschemaandintegrationadapterboundary.
Accessibility NFR-08 Observability NFR-09 Portability NFR-10 Dataintegrity	Keyboard-operableforms, labels,sufficientcontrastandunderstandablestatusmessages. Structuredtracing,requestIDs,healthchecksandnon-sensitiveoperationalmetrics. Containerizedserviceanddocumentedenvironmentconfiguration. Transactionsandconstraintsprotectcriticaloperations;allcorrectionsareattributable.


14.Reportinganddashboarddefinitions
Metric		Definition/caution
Submittedrecords		Numberofrecordsinbatchesreceivedduringaselectedperiod;distinguishsubmittedfromaccepted.
Acceptedrecords		Recordsthatpassedhardvalidationandwerecommittedaccordingtotheconfiguredworkflow.
Rejectedrecords Warnings/reviewcases		Recordsblockedfromcommitduetoharderrors. Recordsacceptedonlyifpolicypermits,orheldforhumanreview;definestatusesprecisely.
Actualenrolment Intakeutilization		Countofactiveacceptedadmissionsunder theconfiguredrule,notmerelycountofstudentidentities. Applicableactualcountdividedbytherelevantapprovedcapacity;handlezero/unconfiguredcapacityexplicitly.
Intakeexceptions		Caseswherethecalculatedcountexceedsaconfiguredrule,withsourceandruleversion.
Duplicatecandidates		Potentialconflictsrequiringreview;donotequateeveryduplicateflagwithfraud.
Dataqualitytrend Verificationactivity	charts.	Rateofissuesbycategoryovertime;accountforchangesinschema/rules. Requests,outcomes,responsetimesandaccess-deniedevents;avoidunnecessarypersonaldatainaggregate


15.Testingandqualityassurance
15.1Requiredtestlayers
- Unittests:APAARfieldchecks,date/statusrules,duplicatelogic, intakecalculationsanderrormapping.
- Integrationtests:databaseconstraints,batchtransactions,role/scopeenforcementandauditeventcreation.
- APItests:valid/invalidpayloads,pagination,authentication,authorizationandrate-limitbehavior.
- Frontendtests:uploadstates,row-levelerrordisplay,role-dependentnavigationandpermissionerrors.
- Securitytests:cross-institutionaccess,IDORattempts,oversizedfiles,malformedCSV,injectionpayloads, tokenexpiry andexportlimits.
- End-to-enddemotest:uploadaknowndataset,verifyexpectedflags,resolveoneissueandconfirmdashboardcounts update.
15.2Seeddatasetscenarios
Case	Expectedresult	
Validadmissionwithinconfiguredcapacity		Accepted;countedinthecorrectinstitution/programme/year. Rejectedorheldaccordingtoconfiguredhard-validationpolicy;clearreason
MissingAPAARID	shown.	Flaggedasformat-invalidonlyiftheformatspecificationisconfirmed;never
MalformedAPAARID DuplicateAPAARIDinsamebatch		claimauthoritativeverification. Duplicatecandidateidentifiedwithrowreferences.
Historicalwithdrawnadmission	SameIDwithconflictingactiveadmissionsacrossinstitutions Cross-institutionreviewissue;noautomaticfraudverdict.	Notcountedasactiveiftheconfiguredstatusrulesaysso;historyretained.
Admissionsexceedregularintake		Intakeexceptionwithcalculatedcountandconfiguredcapacity. Countedaccordingtotheexplicitlyconfigureddemo/policyrule,notassumedto
Supernumeraryseats	beregularseats.	
Batchfailshalfwaythroughcommit Verifierasksforexcessfields	Unauthorizedinstitutionuserqueriesanotherinstitution unambiguous.	Deniedandloggedwithoutrevealingrestrictedrecorddetails. Transaction/commitpolicypreventssilentpartialsuccess;batchstatusis Fieldsarefiltered/deniedandthedisclosureattemptisauditable.


16.Deliveryplanfora2– 7dayteam
Timebox Deliverable Requirementslock,schema,APIcontract,seed	Definitionofdone AgreedCSVcolumns,ruledefinitions,repositorysetupandone
Day1/first20% data	runninghealthendpoint.
Supabaseschema,EdgeFunctions, ingestionand	Uploadreturnsrow-levelresults;corerulescoveredbyunit
Day1– 2/next30% validation	tests.
	Institutioncanuploadandviewresults;regulatordashboard
Day2– 3/next25% ReactUIanddashboards	reflectsactualseededdata. Restrictedaccessworks;oneissuecanbereviewedwith
Day3– 4/next15% Reviewworkflow,rolechecksandauditevents	traceablehistory. Cleandemorun,knownlimitationsdocumented,nocritical
Remainingtime Testing,deployment,demo,documentation	access-controlbugs.


16.1If thereisonlyonedeveloper
- UseCSVratherthansupportingCSVandXLSXimmediately.
- UseSupabasePostgreSQL,AuthandasmallsetofEdgeFunctions;donotbuildseparatemicroservices.
- Startwithdeterministicrules,notanMLmodel.
- Buildonlytworolesinitially:institutionuserandAICTEadministrator;representverifieraccessasacontrolleddemo extensioniftimeallows.
- Maketheend-to-endhappypathworkbeforevisualpolish.
16.2Iftherearetwodevelopers
- DeveloperA:Supabaseschema/RLSmigrations,EdgeFunctions,ingestion,validationandtestfixtures.
- DeveloperB:ReactUI,uploadwizard,dashboardandSupabase/EdgeFunctionintegration.
- Both:agreeonJSONschemasanderrorcodesearly;jointlytestauthorizationandthefinaldemo.
17.Risks,assumptionsandmitigations
Risk/assumption NoauthorizedliveAPAARAPI access Intakerulesdifferby category/year DuplicateIDsaretreatedasfraud Overcollectionofpersonaldata Unauthorizedcross-institution access CSV/XLSXparsingedgecases	Impact Cannotclaimauthoritativeidentity verification. Falsepositivesorincorrect regulatoryconclusions. Legitimatetransfers/historical recordsmaybewronglyflagged. Privacyandbreachimpact increases. Highconfidentialityrisk. Badrecordsorcrashes. Coreworkflowremains	Mitigation/decisionneeded Useamockadapterclearlylabelledsimulated;confirmofficial integrationroutewiththerelevantauthority. Makerulesconfigurable/versioned;obtainauthoritativepolicy interpretation. Usereviewstatesandadmissionlifecycle;explainevidenceand permitcorrection. Dataminimizationandrole-specificfielddisclosure. Server-sidetenantscoping,negativetestsandauditmonitoring. Filelimits,strictschema,safeparsing,row-levelerror reportingand fuzz/edgetests. Useestablishedcrates,minimalmodules,andavoidadvanced
Backendsetupcomplexityslowsdelivery	incomplete. Lowjudgeconfidenceandriskof	unnecessarydistributedinfrastructure. Rule-basedindicatorsfirst;showevidenceandkeepahuman
AIscoreisnotexplainable Prototypemistakenfor	unfairflags. Overstatedsecurityor integration	reviewerincontrol. Documentdemolimitations,mockservices,securitygapsand
production-ready Unclearownership/retention	claims. Datagovernancecannotbe	productiongates. Resolvecontroller/processorresponsibilitiesandapprovedlifecycle
policy	finalized.	policybeforerealdata.


18.Dependenciesanddecisionstoresolve
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
19.AcceptancecriteriaforMVPsign-off
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
- READMEexplainsSupabasesetup,migrationsandRLS,syntheticseeddata,EdgeFunctiondeployment,testsandfrontendstartup.
- KnownlimitationsandproductiondependenciesaredocumentedinthedemoandPRD.
20. Suggested repository structure
.
├── PRD.md
├── BUILD_PLAN.md
├── README.md
├── .env.example
├── package.json
├── vite.config.ts
├── tsconfig.json
├── index.html
├── src/
│   ├── main.tsx
│   ├── App.tsx
│   ├── lib/supabase.ts
│   ├── api/
│   ├── components/
│   ├── pages/
│   ├── hooks/
│   ├── types/
│   └── tests/
├── supabase/
│   ├── config.toml
│   ├── migrations/
│   ├── seed.sql
│   └── functions/
│       ├── validate-batch/
│       ├── commit-batch/
│       ├── resolve-issue/
│       ├── create-verification/
│       └── enrolment-report/
└── docs/
    ├── api-contract.md
    ├── data-dictionary.md
    ├── demo-script.md
    └── assumptions.md
21. Implementation notes
- Represent domain states with constrained values or PostgreSQL enums/check constraints rather than arbitrary strings.
- Keep validation and intake-calculation functions deterministic and unit-testable without a database where possible.
- Validate inputs in the frontend for usability and again in trusted server-side code; never trust browser-supplied institution IDs or role claims.
- Use PostgreSQL transactions for batch commits and review-state changes. A failed commit must not leave partial or ambiguous data.
- Use database constraints and indexes that match business semantics. A unique index on APAAR ID alone may incorrectly prohibit legitimate historical or multi-context records.
- Bound uploaded files by size and row count. Parse and validate on the server for authoritative results; use the frontend only for an optional preview.
- Apply RLS policies to every table containing institution-scoped data. Test both allowed and denied access using separate user identities.
- Keep Supabase service-role keys and privileged secrets exclusively in server-side Edge Function secrets; never place them in VITE_* variables or commit them.
- Version-control schema and RLS changes with Supabase CLI migrations.
- Run formatting/linting, TypeScript checks, unit tests, RLS integration tests and Edge Function tests before the demo.
- Run dependency and secret scans before sharing a deployment.
- Do not introduce an AI model until rules, labels, data quality and evaluation criteria are defined.
22.Demonarrativeforjudges
Recommended3– 5minutedemonstration:
-
  18. Showtheinstitutiondashboardandapprovedintakeconfiguration.
-
  19. UploadasyntheticCSVcontainingavalidrecord,missingAPAARID,duplicatecandidateandintakeoverrun.
-
  20. Showthevalidationsummaryandopenonerecordtoexplaintheexactruleandevidence.
-
  21. SwitchtotheAICTEroleandshowinstitution-levelmetricsandtheexceptionqueue.
-
  22. Resolveordismissoneissuewithareason;showtheaudittrailandupdateddashboard.
-
  23. Demonstratelimitedverification,ifbuilt,andclearlystatewhetheritusesamockorauthorizedlivesource.
-
  24. Closewiththeimpact:consistentrecords,explainableexceptions,privacy-awareaccessandapathtointegration.
Avoidsaying'AIverifiesAPAAR'ifthesystemonlycheckssyntax.Usepreciselanguagesuchas'formatvalidation', 'duplicate candidatedetection', 'intakerulecheck',and'authorizedexternalverificationadapter'.
23.Futureroadmap
Phase	Scope	Entrycriteria
	CSVupload,deterministicvalidation,duplicateflags, intakedashboard,	AllMVPacceptancecriteriapasson
Prototype		
	roles,audittrail.	syntheticdata.
	Approvedidentityintegration,institutionalAPIadapter,correction	Writtenapprovalfordataaccessand
Pilot		
	workflow,monitoreddeployment,policy-reviewedprivacycontrols.	confirmedbusinessrules.
	SSO,operationalmonitoring,backup/restoretests,loadtesting,	Serviceowner,supportmodel,risk
Productionhardening	incidentresponse,securityassessment,accessibilityandretention	assessmentandservice-leveltargets
	controls.	agreed.
	Backgroundbatchprocessing,advancedanalytics,additional	Reliabledata,measurablebaselineand
Scaleandanalytics		
	integrationsandcarefullyevaluatedanomalyscoring.	governanceformodel/rulechanges.


24.Glossary
Term APAARID Approvedintake Actualenrolment Supernumeraryseat	MeaninginthisPRD Academicidentityidentifier referencedbytheproblemstatement;formatandauthoritativeverificationrules mustcomefromapprovedsources. Authorizedcapacityforadefinedinstitution/programme/academicyear,subjecttotheapplicableofficialrules. Countderivedfromadmissionsthatqualifyundertheconfiguredstatusandcountingrules. Aseatcategoryrequiringseparatepolicy-awaretreatmentintheintakecalculation.
Validationissue	Adetectedruleviolation,inconsistencyorreviewflag;notnecessarilyevidenceofmisconduct.
Authoritativeverification	Acheckagainstanapprovedsourceusinganauthorizedinterface/process,notmerelyalocalformatcheck.
RBAC	Role-basedaccesscontrol;mustbecombinedwithrecord/institutionscopingwhereneeded.
MVP	Minimumviableproduct: thesmallestcompleteworkflowthatdemonstratescorevalue.
Modularmonolith	Onedeployableservicedividedintowell-definedinternalmodules,suitableforasmallteam.


25.Final recommendation
BuildaReact+TypeScript+VitefrontendwithSupabaseasthemanagedbackend:PostgreSQLfordata,Authforidentity,RLSforinstitution-scopedaccessandEdgeFunctionsforprivilegedworkflows. Implementsecure institutionalscoping,CSVingestion,deterministicvalidation,cross-institutionduplicatecandidatedetection,policy-configured intakemonitoring,anAICTEdashboardandanaudit trailbeforeaddingadvancedAI.

Fora2– 7daybuild,themostvaluableproofisacomplete, testedworkflowonsyntheticdata.TreatliveAPAARverification, official intakerules,real institutiondataandproductionsecurityassuranceasexplicitintegration/governancegates.

ThisPRDshouldbereviewedwiththerelevantAICTE/APAARstakeholdersbeforeit isusedasaproductionspecification.The suppliedproblemstatementestablishestheproductdirection,butitdoesnotprovidetheexactAPIcontracts, legalbasis, intake formulas,data-retentionschedule,service-levelobjectivesorproductionsecuritystandards.