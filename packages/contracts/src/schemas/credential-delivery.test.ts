import { contractSchemaMatches } from "../internal.js";
import { describe, expect, it } from "vitest";

import {
  credentialDeliveryBrokerResponseV1Schema,
  credentialDeliveryObservationV1Schema,
  credentialDeliveryProfileV1Schema,
  credentialDeliveryRequestV1Schema,
  type CredentialDeliveryBrokerResponseContract,
  type CredentialDeliveryObservationContract,
  type CredentialDeliveryProfileContract,
  type CredentialDeliveryRequestContract,
} from "./credential-delivery.js";

const harnessRef = { type: "harness", uri: "runx:harness:credential-smoke" } as const;
const hostRef = { type: "host", uri: "runx:host:local" } as const;
const grantRef = { type: "grant", uri: "runx:grant:github-repo-read" } as const;
const credentialRef = { type: "credential", uri: "runx:credential:github-installation-1" } as const;
const redactionPolicyRef = { type: "redaction_policy", uri: "runx:redaction-policy:credentials-v1" } as const;
const deliveryHandleRef = { type: "credential", uri: "runx:credential-delivery-handle:req_cred_1:access_token" } as const;

const profile: CredentialDeliveryProfileContract = {
  schema: "runx.credential_delivery.profile.v1",
  profile_id: "github-provider-api-env",
  provider: "github",
  auth_mode: "oauth",
  purpose: "provider_api",
  delivery_mode: "process_env",
  material_roles: ["access_token"],
  env_bindings: [{
    role: "access_token",
    env_var: "GITHUB_TOKEN",
    required: true,
  }],
  redaction_policy_ref: redactionPolicyRef,
};

const request: CredentialDeliveryRequestContract = {
  schema: "runx.credential_delivery.request.v1",
  request_id: "cred_req_1",
  harness_ref: harnessRef,
  host_ref: hostRef,
  grant_ref: grantRef,
  credential_ref: credentialRef,
  profile_id: "github-provider-api-env",
  provider: "github",
  purpose: "provider_api",
  requested_roles: ["access_token"],
  requested_at: "2026-05-22T00:30:00Z",
};

const response: CredentialDeliveryBrokerResponseContract = {
  schema: "runx.credential_delivery.broker_response.v1",
  response_id: "cred_resp_1",
  request_id: "cred_req_1",
  status: "delivered",
  delivery_mode: "process_env",
  handles: [{
    role: "access_token",
    delivery_handle_ref: deliveryHandleRef,
    env_var: "GITHUB_TOKEN",
  }],
  credential_refs: [credentialRef],
  material_ref_hash: "sha256:4ab3",
  issued_at: "2026-05-22T00:30:01Z",
  expires_at: "2026-05-22T00:40:01Z",
};

const observation: CredentialDeliveryObservationContract = {
  schema: "runx.credential_delivery.observation.v1",
  observation_id: "cred_obs_1",
  request_id: "cred_req_1",
  response_id: "cred_resp_1",
  status: "delivered",
  harness_ref: harnessRef,
  host_ref: hostRef,
  profile_id: "github-provider-api-env",
  provider: "github",
  purpose: "provider_api",
  delivery_mode: "process_env",
  credential_refs: [credentialRef],
  material_ref_hash: "sha256:4ab3",
  delivered_roles: ["access_token"],
  redaction_refs: [redactionPolicyRef],
  observed_at: "2026-05-22T00:30:02Z",
};

describe("credential-delivery schemas", () => {
  it("accepts public credential delivery frames without raw material", () => {
    expect(contractSchemaMatches(credentialDeliveryProfileV1Schema, profile)).toBe(true);
    expect(contractSchemaMatches(credentialDeliveryRequestV1Schema, request)).toBe(true);
    expect(contractSchemaMatches(credentialDeliveryBrokerResponseV1Schema, response)).toBe(true);
    expect(contractSchemaMatches(credentialDeliveryObservationV1Schema, observation)).toBe(true);

    const serialized = JSON.stringify({ profile, request, response, observation });
    expect(serialized).not.toContain("sk-contract-test");
    expect(serialized).not.toContain("super-secret-token");
  });

  it("rejects raw secret-like fields on public frames", () => {
    expect(contractSchemaMatches(credentialDeliveryBrokerResponseV1Schema, {
      ...response,
      access_token: "super-secret-token",
    })).toBe(false);
    expect(contractSchemaMatches(credentialDeliveryObservationV1Schema, {
      ...observation,
      api_key: "sk-contract-test",
    })).toBe(false);
    expect(contractSchemaMatches(credentialDeliveryRequestV1Schema, {
      ...request,
      password: "hunter2",
    })).toBe(false);
  });

  it("rejects non-env delivery modes until a future contract explicitly adds them", () => {
    expect(contractSchemaMatches(credentialDeliveryProfileV1Schema, {
      ...profile,
      delivery_mode: "file",
    })).toBe(false);
  });
});
