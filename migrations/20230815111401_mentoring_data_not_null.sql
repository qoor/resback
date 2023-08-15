-- Add migration script here
ALTER TABLE `senior_users` MODIFY COLUMN `mentoring_method_id` int(10) unsigned NOT NULL DEFAULT 1;
ALTER TABLE `senior_users` MODIFY COLUMN `mentoring_status` bool NOT NULL DEFAULT FALSE;
ALTER TABLE `senior_users` MODIFY COLUMN `mentoring_always_on` bool NOT NULL DEFAULT FALSE;
