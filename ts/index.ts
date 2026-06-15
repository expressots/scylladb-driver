import type { QueryResult as NativeQueryResult } from "../index.js";
import {
  Cluster,
  ScyllaBatchStatement,
  ScyllaPreparedStatement,
  ScyllaSession,
  Uuid,
} from "../index.js";

export {
  Cluster,
  ScyllaSession,
  ScyllaPreparedStatement,
  ScyllaBatchStatement,
  Uuid,
  initLogging,
} from "../index.js";

export type {
  AddressTranslationEntry,
  AttemptHistoryInfo,
  BatchStatement,
  ClusterConfig,
  ColumnInfo,
  ColumnSpec,
  DriverMetrics,
  ExecuteOptions,
  ExecutionProfileConfig,
  KeyspaceInfo,
  MaterializedViewInfo,
  PagedQueryResult,
  PercentileSpeculativeConfig,
  QueryResult,
  QueryTracingInfo,
  QueryWithHistory,
  RequestHistoryInfo,
  ScyllaCluster,
  SpeculativeExecutionConfig,
  TableInfo,
  TlsConfig,
  TracingEvent,
  UdtFieldInfo,
  UserDefinedTypeInfo,
} from "../index.js";

export type Row = Record<string, unknown>;

export interface TypedQueryResult<T extends Row = Row>
  extends Omit<NativeQueryResult, "rows"> {
  rows: T[];
}

export function getRows(result: NativeQueryResult): Row[] {
  return result.rows as Row[];
}

export function getFirstRow(result: NativeQueryResult): Row | undefined {
  return getRows(result)[0];
}
