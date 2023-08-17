-- Add migration script here
ALTER TABLE `senior_users` MODIFY COLUMN `mentoring_price` int(10) unsigned NOT NULL;
