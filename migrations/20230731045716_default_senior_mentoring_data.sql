-- Add migration script here
ALTER TABLE `senior_users` MODIFY COLUMN `mentoring_method_id` int(10) unsigned DEFAULT 2;
ALTER TABLE `senior_users` MODIFY COLUMN `mentoring_status` bool DEFAULT false;
ALTER TABLE `senior_users` MODIFY COLUMN `mentoring_always_on` bool DEFAULT false;
