-- Add migration script here
DROP TABLE IF EXISTS `mentoring_order`;
CREATE TABLE `mentroing_order` (
  `id` int(10) unsigned NOT NULL AUTO_INCREMENT,
  `buyer_id` int(10) unsigned NOT NULL,
  `seller_id` int(10) unsigned,
  `mentoring_time_id` int(10) unsigned NOT NULL,
  `mentoring_method_id` int(10) unsigned NOT NULL,
  `mentoring_price` int(10) unsigned NOT NULL,
  `mentoring_content` varchar(1024) NOT NULL,
  `created_at` timestamp NOT NULL DEFAULT current_timestamp(),
  PRIMARY KEY (`id`),
  CONSTRAINT `mentoring_order_buyer` FOREIGN KEY (`buyer_id`) REFERENCES `normal_users` (`id`) ON DELETE CASCADE,
  CONSTRAINT `mentoring_order_seller` FOREIGN KEY (`seller_id`) REFERENCES `senior_users` (`id`) ON DELETE SET NULL,
  CONSTRAINT `mentoring_order_time` FOREIGN KEY (`mentoring_time_id`) REFERENCES `mentoring_time` (`id`),
  CONSTRAINT `mentoring_order_method` FOREIGN KEY (`mentoring_method_id`) REFERENCES `mentoring_method` (`id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci;
