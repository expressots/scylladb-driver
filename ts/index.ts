/**
 * TypeScript helpers and re-exports for `scylladb-driver`.
 *
 * Import from `scylladb-driver/ts` for typed row helpers alongside the native API.
 *
 * @packageDocumentation
 */
import type { QueryResult as NativeQueryResult } from "../index.js";

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

/** A single CQL result row as a plain object keyed by column name. */
export type Row = Record<string, unknown>;

/**
 * {@link QueryResult} with rows typed as `T` instead of `any`.
 *
 * @typeParam T - Shape of each row object.
 */
export interface TypedQueryResult<T extends Row = Row>
  extends Omit<NativeQueryResult, "rows"> {
  rows: T[];
}

/**
 * Returns result rows cast to plain objects.
 *
 * @param result - Query result from {@link ScyllaSession.execute} or similar.
 * @returns Array of row objects keyed by column name.
 */
export function getRows(result: NativeQueryResult): Row[] {
  return result.rows as Row[];
}

/**
 * Returns the first row of a query result, if any.
 *
 * @param result - Query result from {@link ScyllaSession.execute} or similar.
 * @returns First row or `undefined` when the result set is empty.
 */
export function getFirstRow(result: NativeQueryResult): Row | undefined {
  return getRows(result)[0];
}
