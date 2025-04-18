#ifndef RNS_H
#define RNS_H

#ifdef __cplusplus
extern "C" {
#endif

// Neovim API functions
extern int nvim_set_option_bool(const char *name, int value);
extern int nvim_set_option_int(const char *name, int value);
extern int nvim_set_option_string(const char *name, const char *value);
extern int nvim_set_global(const char *name, const char *value);
extern int nvim_create_keymap(const char *mode, const char *lhs,
                              const char *rhs, const char *opts);
extern int nvim_create_user_command(const char *name, const char *command,
                                    const char *opts);
extern int nvim_create_autocmd(const char *event, const char *pattern,
                               const char *command, const char *group);
extern int nvim_create_augroup(const char *name, int clear);
extern int nvim_buf_set_keymap(int buffer, const char *mode, const char *lhs,
                               const char *rhs, const char *opts);
extern int nvim_exec_command(const char *command);

// Enhanced Lua API for autocmds
extern int nvim_create_augroup_lua(const char *name, int clear);
extern int nvim_create_autocmd_lua(const char *event, const char *pattern,
                                   const char *command, const char *group);

// Legacy functions
extern int opt(const char *key, const char *old_val, const char *new_val);
extern int autocmd(const char *event, const char *pattern, const char *command);
extern int exec_lua(const char *code);
extern int setup_lsp(const char *server, const char *config_json);

// Plugin manager
extern int register_plugin(const char *name, const char *url);
extern int configure_plugin(const char *name, const char *config);
extern int install_plugins(void);
extern int load_plugin_configs(void);
extern int update_plugins(void);

// Structured plugin configuration API
extern int plugin_config_begin(const char *plugin_name);
extern int plugin_config_end(void);
extern int plugin_config_add_server(const char *server_name);
extern int plugin_config_set_server_option(const char *server,
                                           const char *option,
                                           const char *value);
extern int plugin_config_set_mapping(const char *plugin, const char *mode,
                                     const char *key, const char *action);
extern int plugin_config_add_keymap(const char *mode, const char *key,
                                    const char *plugin, const char *command);

#ifdef __cplusplus
}
#endif

#endif // RNS_H
