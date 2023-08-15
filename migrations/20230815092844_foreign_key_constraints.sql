-- Add migration script here
ALTER TABLE `mentoring_schedule` DROP FOREIGN KEY `mentoring_schedule_ibfk_1`;
ALTER TABLE `email_verification` DROP FOREIGN KEY `email_verification_ibfk_1`;

ALTER TABLE `mentoring_schedule` ADD CONSTRAINT `mentoring_schedule_user` FOREIGN KEY (`senior_id`) REFERENCES `senior_users` (`id`) ON DELETE CASCADE;
ALTER TABLE `email_verification` ADD CONSTRAINT `email_verification_user` FOREIGN KEY (`senior_id`) REFERENCES `senior_users` (`id`) ON DELETE CASCADE;
