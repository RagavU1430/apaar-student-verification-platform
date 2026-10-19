-- Demo master data. Synthetic only - no real institution, student or approval
-- record is reproduced here. Source references are labelled DEMO-SEED-v1
-- because no official AICTE approval source has been confirmed
-- (BUILD_PLAN.md A-03).

INSERT INTO institutions (id, official_code, name, status) VALUES
    ('11111111-1111-4111-8111-111111111111', 'DEMO-INST-001', 'Alpha Institute of Technology', 'ACTIVE'),
    ('22222222-2222-4222-8222-222222222222', 'DEMO-INST-002', 'Beta Institute of Engineering', 'ACTIVE');

INSERT INTO programmes (id, institution_id, code, name, level, status) VALUES
    ('aaaaaaaa-0000-4000-8000-000000000001', '11111111-1111-4111-8111-111111111111', 'CSE-01', 'B.Tech Computer Science and Engineering', 'UG', 'ACTIVE'),
    ('aaaaaaaa-0000-4000-8000-000000000002', '11111111-1111-4111-8111-111111111111', 'ECE-01', 'B.Tech Electronics and Communication', 'UG', 'ACTIVE'),
    ('bbbbbbbb-0000-4000-8000-000000000001', '22222222-2222-4222-8222-222222222222', 'MECH-01', 'B.Tech Mechanical Engineering', 'UG', 'ACTIVE');

INSERT INTO approved_intakes
    (id, programme_id, academic_year, regular_intake, supernumerary_capacity, source_reference, rule_version, effective_version)
VALUES
    ('cccccccc-0000-4000-8000-000000000001', 'aaaaaaaa-0000-4000-8000-000000000001', '2026-27', 60, 6, 'DEMO-SEED-v1', 'demo-v1', 1),
    ('cccccccc-0000-4000-8000-000000000002', 'aaaaaaaa-0000-4000-8000-000000000002', '2026-27', 40, 4, 'DEMO-SEED-v1', 'demo-v1', 1),
    ('cccccccc-0000-4000-8000-000000000003', 'bbbbbbbb-0000-4000-8000-000000000001', '2026-27', 45, 4, 'DEMO-SEED-v1', 'demo-v1', 1);

INSERT INTO intake_rules (id, rule_version, seat_category, counts_toward_regular, uses_supernumerary_pool, notes) VALUES
    ('dddddddd-0000-4000-8000-000000000001', 'demo-v1', 'REGULAR', TRUE, FALSE, 'Counts against the regular approved intake.'),
    ('dddddddd-0000-4000-8000-000000000002', 'demo-v1', 'SUPERNUMERARY', FALSE, TRUE, 'Counts against the separately configured supernumerary pool.'),
    ('dddddddd-0000-4000-8000-000000000003', 'demo-v1', 'NRI', FALSE, TRUE, 'Demo rule only. Official category treatment is unconfirmed.'),
    ('dddddddd-0000-4000-8000-000000000004', 'demo-v1', 'OCI', FALSE, TRUE, 'Demo rule only. Official category treatment is unconfirmed.'),
    ('dddddddd-0000-4000-8000-000000000005', 'demo-v1', 'FOREIGN_NATIONAL', FALSE, TRUE, 'Demo rule only. Official category treatment is unconfirmed.');
