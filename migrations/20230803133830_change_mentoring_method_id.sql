-- Add migration script here
SET FOREIGN_KEY_CHECKS=0;

DELETE FROM `mentoring_method`;
INSERT INTO `mentoring_method` (`id`, `name`) VALUES
  (1, 'video_call'),
  (2, 'voice_call')
;

ALTER TABLE `senior_users` MODIFY COLUMN `mentoring_method_id` int(10) unsigned DEFAULT 1;

SET FOREIGN_KEY_CHECKS=1;
