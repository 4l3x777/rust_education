#ifndef SMART_SOCKET_H
#define SMART_SOCKET_H

/*
 * ffi_smart_socket.h — C-заголовок для FFI-обёртки SmartSocket.
 *
 * Соответствует экспортам ffi_smart_socket (Rust staticlib / cdylib).
 *
 * Коды возврата целочисленных функций:
 *   0   — успех (OK)
 *  -1   — передан NULL-указатель (ERR_NULL)
 *  -2   — операция завершилась ошибкой (ERR_OP)
 *
 * socket_is_on:
 *   1   — розетка включена
 *   0   — розетка выключена
 *  -1   — NULL-указатель
 *
 * socket_power:
 *  >= 0 — текущая мощность (Вт)
 *  -1.0 — NULL-указатель или ошибка
 */

#ifdef __cplusplus
extern "C" {
#endif

#include <stddef.h>

/* Непрозрачный тип — C-код не заглядывает внутрь. */
typedef struct SmartSocket SmartSocket;

/**
 * Создать розетку на куче.
 * @param name        Имя устройства (UTF-8, null-terminated).
 * @param is_on       Начальное состояние: 1 = включена, 0 = выключена.
 * @param power_watts Номинальная мощность (Вт).
 * @return Указатель на розетку, или NULL при ошибке.
 *         Память освобождается только через socket_destroy().
 */
SmartSocket *socket_create(const char *name, int is_on, double power_watts);

/**
 * Включить розетку.
 * @return 0 — успех; -1 — NULL ptr; -2 — внутренняя ошибка.
 */
int socket_turn_on(SmartSocket *socket);

/**
 * Выключить розетку.
 * @return 0 — успех; -1 — NULL ptr; -2 — внутренняя ошибка.
 */
int socket_turn_off(SmartSocket *socket);

/**
 * Запросить состояние (включена / выключена).
 * @return 1 — включена; 0 — выключена; -1 — NULL ptr.
 */
int socket_is_on(const SmartSocket *socket);

/**
 * Запросить текущую мощность (Вт).
 * Возвращает 0.0, когда розетка выключена.
 * @return Мощность в Вт, или -1.0 при NULL ptr / ошибке.
 */
double socket_power(const SmartSocket *socket);

/**
 * Освободить память, выделенную socket_create().
 * NULL-safe: вызов с NULL не приводит к ошибке.
 */
void socket_destroy(SmartSocket *socket);

#ifdef __cplusplus
}
#endif

#endif /* SMART_SOCKET_H */
