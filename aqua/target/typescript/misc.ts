/* eslint-disable */
// @ts-nocheck
/**
 *
 * This file is auto-generated. Do not edit manually: changes may be erased.
 * Generated by Aqua compiler: https://github.com/fluencelabs/aqua/.
 * If you find any bugs, please write an issue on GitHub: https://github.com/fluencelabs/aqua/issues
 * Aqua version: 0.11.9-release-please-1c9388a-1275-1
 *
 */
import type { IFluenceClient as IFluenceClient$$, CallParams as CallParams$$ } from '@fluencelabs/js-client.api';
import {
    v5_callFunction as callFunction$$,
    v5_registerService as registerService$$,
} from '@fluencelabs/js-client.api';
    


// Services

// Functions
export const getResourceHelper_script = `
                    (seq
                     (seq
                      (seq
                       (seq
                        (call %init_peer_id% ("getDataSrv" "-relay-") [] -relay-)
                        (call %init_peer_id% ("getDataSrv" "resource_id") [] resource_id)
                       )
                       (xor
                        (new $resources
                         (new $successful
                          (new $result
                           (seq
                            (seq
                             (seq
                              (seq
                               (seq
                                (seq
                                 (call %init_peer_id% ("op" "string_to_b58") [resource_id] k)
                                 (call %init_peer_id% ("kad" "neighborhood") [k [] []] nodes)
                                )
                                (par
                                 (fold nodes n-0
                                  (par
                                   (xor
                                    (seq
                                     (new $-ephemeral-stream-
                                      (new #-ephemeral-canon-
                                       (canon -relay- $-ephemeral-stream-  #-ephemeral-canon-)
                                      )
                                     )
                                     (xor
                                      (seq
                                       (call n-0 ("registry" "get_key_metadata") [resource_id] get_result)
                                       (xor
                                        (seq
                                         (seq
                                          (match get_result.$.success true
                                           (seq
                                            (ap get_result.$.key $resources)
                                            (ap true $successful)
                                           )
                                          )
                                          (new $-ephemeral-stream-
                                           (new #-ephemeral-canon-
                                            (canon -relay- $-ephemeral-stream-  #-ephemeral-canon-)
                                           )
                                          )
                                         )
                                         (new $-ephemeral-stream-
                                          (new #-ephemeral-canon-
                                           (canon %init_peer_id% $-ephemeral-stream-  #-ephemeral-canon-)
                                          )
                                         )
                                        )
                                        (seq
                                         (seq
                                          (seq
                                           (call n-0 ("op" "concat_strings") [get_result.$.error " on "] e)
                                           (call n-0 ("op" "concat_strings") [e n-0] $error)
                                          )
                                          (new $-ephemeral-stream-
                                           (new #-ephemeral-canon-
                                            (canon -relay- $-ephemeral-stream-  #-ephemeral-canon-)
                                           )
                                          )
                                         )
                                         (new $-ephemeral-stream-
                                          (new #-ephemeral-canon-
                                           (canon %init_peer_id% $-ephemeral-stream-  #-ephemeral-canon-)
                                          )
                                         )
                                        )
                                       )
                                      )
                                      (null)
                                     )
                                    )
                                    (seq
                                     (seq
                                      (new $-ephemeral-stream-
                                       (new #-ephemeral-canon-
                                        (canon -relay- $-ephemeral-stream-  #-ephemeral-canon-)
                                       )
                                      )
                                      (new $-ephemeral-stream-
                                       (new #-ephemeral-canon-
                                        (canon %init_peer_id% $-ephemeral-stream-  #-ephemeral-canon-)
                                       )
                                      )
                                     )
                                     (fail %last_error%)
                                    )
                                   )
                                   (next n-0)
                                  )
                                  (never)
                                 )
                                 (null)
                                )
                               )
                               (new $status
                                (new $result-0
                                 (seq
                                  (seq
                                   (seq
                                    (par
                                     (seq
                                      (seq
                                       (seq
                                        (call %init_peer_id% ("math" "sub") [1 1] sub)
                                        (new $successful_test
                                         (seq
                                          (seq
                                           (seq
                                            (call %init_peer_id% ("math" "add") [sub 1] successful_incr)
                                            (fold $successful successful_fold_var
                                             (seq
                                              (seq
                                               (ap successful_fold_var $successful_test)
                                               (canon %init_peer_id% $successful_test  #successful_iter_canon)
                                              )
                                              (xor
                                               (match #successful_iter_canon.length successful_incr
                                                (null)
                                               )
                                               (next successful_fold_var)
                                              )
                                             )
                                             (never)
                                            )
                                           )
                                           (canon %init_peer_id% $successful_test  #successful_result_canon)
                                          )
                                          (ap #successful_result_canon successful_gate)
                                         )
                                        )
                                       )
                                       (call %init_peer_id% ("math" "sub") [1 1] sub-0)
                                      )
                                      (ap "ok" $status)
                                     )
                                     (call %init_peer_id% ("peer" "timeout") [6000 "timeout"] $status)
                                    )
                                    (new $status_test
                                     (seq
                                      (seq
                                       (seq
                                        (call %init_peer_id% ("math" "add") [0 1] status_incr)
                                        (fold $status status_fold_var
                                         (seq
                                          (seq
                                           (ap status_fold_var $status_test)
                                           (canon %init_peer_id% $status_test  #status_iter_canon)
                                          )
                                          (xor
                                           (match #status_iter_canon.length status_incr
                                            (null)
                                           )
                                           (next status_fold_var)
                                          )
                                         )
                                         (never)
                                        )
                                       )
                                       (canon %init_peer_id% $status_test  #status_result_canon)
                                      )
                                      (ap #status_result_canon status_gate)
                                     )
                                    )
                                   )
                                   (xor
                                    (match status_gate.$.[0] "ok"
                                     (ap true $result-0)
                                    )
                                    (ap false $result-0)
                                   )
                                  )
                                  (new $result-0_test
                                   (seq
                                    (seq
                                     (seq
                                      (call %init_peer_id% ("math" "add") [0 1] result-0_incr)
                                      (fold $result-0 result-0_fold_var
                                       (seq
                                        (seq
                                         (ap result-0_fold_var $result-0_test)
                                         (canon %init_peer_id% $result-0_test  #result-0_iter_canon)
                                        )
                                        (xor
                                         (match #result-0_iter_canon.length result-0_incr
                                          (null)
                                         )
                                         (next result-0_fold_var)
                                        )
                                       )
                                       (never)
                                      )
                                     )
                                     (canon %init_peer_id% $result-0_test  #result-0_result_canon)
                                    )
                                    (ap #result-0_result_canon result-0_gate)
                                   )
                                  )
                                 )
                                )
                               )
                              )
                              (xor
                               (match result-0_gate.$.[0] false
                                (ap "resource not found: timeout exceeded" $error)
                               )
                               (seq
                                (seq
                                 (canon %init_peer_id% $resources  #resources_canon)
                                 (call %init_peer_id% ("registry" "merge_keys") [#resources_canon] merge_result)
                                )
                                (xor
                                 (match merge_result.$.success true
                                  (ap merge_result.$.key $result)
                                 )
                                 (ap merge_result.$.error $error)
                                )
                               )
                              )
                             )
                             (canon %init_peer_id% $result  #-result-fix-0)
                            )
                            (ap #-result-fix-0 -result-flat-0)
                           )
                          )
                         )
                        )
                        (call %init_peer_id% ("errorHandlingSrv" "error") [%last_error% 0])
                       )
                      )
                      (canon %init_peer_id% $error  #error_canon)
                     )
                     (call %init_peer_id% ("callbackSrv" "response") [-result-flat-0 #error_canon])
                    )
    `
 
export type GetResourceHelperResult = [{ challenge: number[]; challenge_type: string; id: string; label: string; owner_peer_id: string; signature: number[]; timestamp_created: number; } | null, string[]]
export function getResourceHelper(
    resource_id: string,
    config?: {ttl?: number}
): Promise<GetResourceHelperResult>;

export function getResourceHelper(
    peer: IFluenceClient$$,
    resource_id: string,
    config?: {ttl?: number}
): Promise<GetResourceHelperResult>;

export function getResourceHelper(...args: any) {


    return callFunction$$(
        args,
        {
    "functionName" : "getResourceHelper",
    "arrow" : {
        "tag" : "arrow",
        "domain" : {
            "tag" : "labeledProduct",
            "fields" : {
                "resource_id" : {
                    "tag" : "scalar",
                    "name" : "string"
                }
            }
        },
        "codomain" : {
            "tag" : "unlabeledProduct",
            "items" : [
                {
                    "tag" : "option",
                    "type" : {
                        "tag" : "struct",
                        "name" : "Key",
                        "fields" : {
                            "challenge" : {
                                "tag" : "array",
                                "type" : {
                                    "tag" : "scalar",
                                    "name" : "u8"
                                }
                            },
                            "label" : {
                                "tag" : "scalar",
                                "name" : "string"
                            },
                            "signature" : {
                                "tag" : "array",
                                "type" : {
                                    "tag" : "scalar",
                                    "name" : "u8"
                                }
                            },
                            "id" : {
                                "tag" : "scalar",
                                "name" : "string"
                            },
                            "owner_peer_id" : {
                                "tag" : "scalar",
                                "name" : "string"
                            },
                            "challenge_type" : {
                                "tag" : "scalar",
                                "name" : "string"
                            },
                            "timestamp_created" : {
                                "tag" : "scalar",
                                "name" : "u64"
                            }
                        }
                    }
                },
                {
                    "tag" : "array",
                    "type" : {
                        "tag" : "scalar",
                        "name" : "string"
                    }
                }
            ]
        }
    },
    "names" : {
        "relay" : "-relay-",
        "getDataSrv" : "getDataSrv",
        "callbackSrv" : "callbackSrv",
        "responseSrv" : "callbackSrv",
        "responseFnName" : "response",
        "errorHandlingSrv" : "errorHandlingSrv",
        "errorFnName" : "error"
    }
},
        getResourceHelper_script
    )
}

export const appendErrors_script = `
                    (seq
                     (seq
                      (seq
                       (seq
                        (seq
                         (call %init_peer_id% ("getDataSrv" "-relay-") [] -relay-)
                         (call %init_peer_id% ("getDataSrv" "error1") [] error1-iter)
                        )
                        (fold error1-iter error1-item-0
                         (seq
                          (ap error1-item-0 $error1)
                          (next error1-item-0)
                         )
                        )
                       )
                       (call %init_peer_id% ("getDataSrv" "error2") [] error2-iter)
                      )
                      (fold error2-iter error2-item-0
                       (seq
                        (ap error2-item-0 $error2)
                        (next error2-item-0)
                       )
                      )
                     )
                     (xor
                      (seq
                       (canon %init_peer_id% $error2  #error2_canon)
                       (fold #error2_canon e-0
                        (seq
                         (ap e-0 $error1-0)
                         (next e-0)
                        )
                       )
                      )
                      (call %init_peer_id% ("errorHandlingSrv" "error") [%last_error% 0])
                     )
                    )
    `
 

export function appendErrors(
    error1: string[],
    error2: string[],
    config?: {ttl?: number}
): Promise<void>;

export function appendErrors(
    peer: IFluenceClient$$,
    error1: string[],
    error2: string[],
    config?: {ttl?: number}
): Promise<void>;

export function appendErrors(...args: any) {


    return callFunction$$(
        args,
        {
    "functionName" : "appendErrors",
    "arrow" : {
        "tag" : "arrow",
        "domain" : {
            "tag" : "labeledProduct",
            "fields" : {
                "error1" : {
                    "tag" : "array",
                    "type" : {
                        "tag" : "scalar",
                        "name" : "string"
                    }
                },
                "error2" : {
                    "tag" : "array",
                    "type" : {
                        "tag" : "scalar",
                        "name" : "string"
                    }
                }
            }
        },
        "codomain" : {
            "tag" : "nil"
        }
    },
    "names" : {
        "relay" : "-relay-",
        "getDataSrv" : "getDataSrv",
        "callbackSrv" : "callbackSrv",
        "responseSrv" : "callbackSrv",
        "responseFnName" : "response",
        "errorHandlingSrv" : "errorHandlingSrv",
        "errorFnName" : "error"
    }
},
        appendErrors_script
    )
}

export const getNeighbors_script = `
                    (seq
                     (seq
                      (seq
                       (call %init_peer_id% ("getDataSrv" "-relay-") [] -relay-)
                       (call %init_peer_id% ("getDataSrv" "resource_id") [] resource_id)
                      )
                      (xor
                       (seq
                        (call %init_peer_id% ("op" "string_to_b58") [resource_id] k)
                        (call %init_peer_id% ("kad" "neighborhood") [k [] []] nodes)
                       )
                       (call %init_peer_id% ("errorHandlingSrv" "error") [%last_error% 0])
                      )
                     )
                     (call %init_peer_id% ("callbackSrv" "response") [nodes])
                    )
    `
 

export function getNeighbors(
    resource_id: string,
    config?: {ttl?: number}
): Promise<string[]>;

export function getNeighbors(
    peer: IFluenceClient$$,
    resource_id: string,
    config?: {ttl?: number}
): Promise<string[]>;

export function getNeighbors(...args: any) {


    return callFunction$$(
        args,
        {
    "functionName" : "getNeighbors",
    "arrow" : {
        "tag" : "arrow",
        "domain" : {
            "tag" : "labeledProduct",
            "fields" : {
                "resource_id" : {
                    "tag" : "scalar",
                    "name" : "string"
                }
            }
        },
        "codomain" : {
            "tag" : "unlabeledProduct",
            "items" : [
                {
                    "tag" : "array",
                    "type" : {
                        "tag" : "scalar",
                        "name" : "string"
                    }
                }
            ]
        }
    },
    "names" : {
        "relay" : "-relay-",
        "getDataSrv" : "getDataSrv",
        "callbackSrv" : "callbackSrv",
        "responseSrv" : "callbackSrv",
        "responseFnName" : "response",
        "errorHandlingSrv" : "errorHandlingSrv",
        "errorFnName" : "error"
    }
},
        getNeighbors_script
    )
}

export const wait_script = `
                    (seq
                     (seq
                      (seq
                       (seq
                        (seq
                         (seq
                          (call %init_peer_id% ("getDataSrv" "-relay-") [] -relay-)
                          (call %init_peer_id% ("getDataSrv" "successful") [] successful-iter)
                         )
                         (fold successful-iter successful-item-0
                          (seq
                           (ap successful-item-0 $successful)
                           (next successful-item-0)
                          )
                         )
                        )
                        (call %init_peer_id% ("getDataSrv" "len") [] len)
                       )
                       (call %init_peer_id% ("getDataSrv" "timeout") [] timeout)
                      )
                      (xor
                       (new $status
                        (new $result
                         (seq
                          (seq
                           (seq
                            (par
                             (seq
                              (seq
                               (seq
                                (call %init_peer_id% ("math" "sub") [len 1] sub)
                                (new $successful_test
                                 (seq
                                  (seq
                                   (seq
                                    (call %init_peer_id% ("math" "add") [sub 1] successful_incr)
                                    (fold $successful successful_fold_var
                                     (seq
                                      (seq
                                       (ap successful_fold_var $successful_test)
                                       (canon %init_peer_id% $successful_test  #successful_iter_canon)
                                      )
                                      (xor
                                       (match #successful_iter_canon.length successful_incr
                                        (null)
                                       )
                                       (next successful_fold_var)
                                      )
                                     )
                                     (never)
                                    )
                                   )
                                   (canon %init_peer_id% $successful_test  #successful_result_canon)
                                  )
                                  (ap #successful_result_canon successful_gate)
                                 )
                                )
                               )
                               (call %init_peer_id% ("math" "sub") [len 1] sub-0)
                              )
                              (ap "ok" $status)
                             )
                             (call %init_peer_id% ("peer" "timeout") [timeout "timeout"] $status)
                            )
                            (new $status_test
                             (seq
                              (seq
                               (seq
                                (call %init_peer_id% ("math" "add") [0 1] status_incr)
                                (fold $status status_fold_var
                                 (seq
                                  (seq
                                   (ap status_fold_var $status_test)
                                   (canon %init_peer_id% $status_test  #status_iter_canon)
                                  )
                                  (xor
                                   (match #status_iter_canon.length status_incr
                                    (null)
                                   )
                                   (next status_fold_var)
                                  )
                                 )
                                 (never)
                                )
                               )
                               (canon %init_peer_id% $status_test  #status_result_canon)
                              )
                              (ap #status_result_canon status_gate)
                             )
                            )
                           )
                           (xor
                            (match status_gate.$.[0] "ok"
                             (ap true $result)
                            )
                            (ap false $result)
                           )
                          )
                          (new $result_test
                           (seq
                            (seq
                             (seq
                              (call %init_peer_id% ("math" "add") [0 1] result_incr)
                              (fold $result result_fold_var
                               (seq
                                (seq
                                 (ap result_fold_var $result_test)
                                 (canon %init_peer_id% $result_test  #result_iter_canon)
                                )
                                (xor
                                 (match #result_iter_canon.length result_incr
                                  (null)
                                 )
                                 (next result_fold_var)
                                )
                               )
                               (never)
                              )
                             )
                             (canon %init_peer_id% $result_test  #result_result_canon)
                            )
                            (ap #result_result_canon result_gate)
                           )
                          )
                         )
                        )
                       )
                       (call %init_peer_id% ("errorHandlingSrv" "error") [%last_error% 0])
                      )
                     )
                     (call %init_peer_id% ("callbackSrv" "response") [result_gate.$.[0]])
                    )
    `
 

export function wait(
    successful: boolean[],
    len: number,
    timeout: number,
    config?: {ttl?: number}
): Promise<boolean>;

export function wait(
    peer: IFluenceClient$$,
    successful: boolean[],
    len: number,
    timeout: number,
    config?: {ttl?: number}
): Promise<boolean>;

export function wait(...args: any) {


    return callFunction$$(
        args,
        {
    "functionName" : "wait",
    "arrow" : {
        "tag" : "arrow",
        "domain" : {
            "tag" : "labeledProduct",
            "fields" : {
                "successful" : {
                    "tag" : "array",
                    "type" : {
                        "tag" : "scalar",
                        "name" : "bool"
                    }
                },
                "len" : {
                    "tag" : "scalar",
                    "name" : "i16"
                },
                "timeout" : {
                    "tag" : "scalar",
                    "name" : "u16"
                }
            }
        },
        "codomain" : {
            "tag" : "unlabeledProduct",
            "items" : [
                {
                    "tag" : "scalar",
                    "name" : "bool"
                }
            ]
        }
    },
    "names" : {
        "relay" : "-relay-",
        "getDataSrv" : "getDataSrv",
        "callbackSrv" : "callbackSrv",
        "responseSrv" : "callbackSrv",
        "responseFnName" : "response",
        "errorHandlingSrv" : "errorHandlingSrv",
        "errorFnName" : "error"
    }
},
        wait_script
    )
}

/* eslint-enable */