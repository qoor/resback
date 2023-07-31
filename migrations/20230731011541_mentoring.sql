-- Add migration script here

ALTER TABLE `senior_users` ADD COLUMN `mentoring_method_id` int(10) unsigned AFTER `description`;
ALTER TABLE `senior_users` ADD COLUMN `mentoring_status` bool AFTER `mentoring_method_id`;
ALTER TABLE `senior_users` ADD COLUMN `mentoring_always_on` bool AFTER `mentoring_status`;

DROP TABLE IF EXISTS `mentoring_method`;
CREATE TABLE `mentoring_method` (
  `id` int(10) unsigned NOT NULL AUTO_INCREMENT,
  `name` varchar(16) NOT NULL,
  PRIMARY KEY (`id`),
  UNIQUE KEY `unique_index` (`name`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci;

DROP TABLE IF EXISTS `mentoring_time`;
CREATE TABLE `mentoring_time` (
  `id` int(10) unsigned NOT NULL AUTO_INCREMENT,
  `hour` int(10) unsigned NOT NULL,
  PRIMARY KEY (`id`),
  UNIQUE KEY `unique_index` (`hour`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci;

DROP TABLE IF EXISTS `mentoring_schedule`;
CREATE TABLE `mentoring_schedule` (
  `id` int(10) unsigned NOT NULL AUTO_INCREMENT,
  `senior_id` int(10) unsigned NOT NULL,
  `time_id` int(10) unsigned NOT NULL,
  PRIMARY KEY (`id`),
  FOREIGN KEY (`senior_id`) REFERENCES `senior_users` (`id`),
  FOREIGN KEY (`time_id`) REFERENCES `mentoring_time` (`id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci;

ALTER TABLE `senior_users` ADD FOREIGN KEY (`mentoring_method_id`) REFERENCES `mentoring_method` (`id`);
