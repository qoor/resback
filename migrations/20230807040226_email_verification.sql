-- Add migration script here

DROP TABLE IF EXISTS `email_verification`;
CREATE TABLE `email_verification` (
  `id` int(10) unsigned NOT NULL AUTO_INCREMENT,
  `senior_id` int(10) unsigned NOT NULL,
  `code` varchar(16) NOT NULL,
  `created_at` timestamp NOT NULL DEFAULT current_timestamp(),
  PRIMARY KEY (`id`),
  UNIQUE KEY `unique_index` (`senior_id`),
  FOREIGN KEY (`senior_id`) REFERENCES `senior_users` (`id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci;

ALTER TABLE `senior_users` ADD COLUMN `email_verified` bool NOT NULL DEFAULT false AFTER `mentoring_always_on`;
