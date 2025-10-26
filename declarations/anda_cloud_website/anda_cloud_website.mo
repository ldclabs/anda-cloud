// This is a generated Motoko binding.
// Please use `import service "ic:canister_id"` instead to call canisters on the IC if possible.

module {
  public type AssetCanisterArgs = { #Upgrade : UpgradeArgs; #Init : InitArgs };
  public type BatchId = Nat;
  public type BatchOperationKind = {
    #SetAssetProperties : SetAssetPropertiesArguments;
    #CreateAsset : CreateAssetArguments;
    #UnsetAssetContent : UnsetAssetContentArguments;
    #DeleteAsset : DeleteAssetArguments;
    #SetAssetContent : SetAssetContentArguments;
    #Clear : ClearArguments;
  };
  public type ChunkId = Nat;
  /// Reset everything
  public type ClearArguments = {};
  public type CommitBatchArguments = {
    batch_id : BatchId;
    operations : [BatchOperationKind];
  };
  public type CommitProposedBatchArguments = {
    batch_id : BatchId;
    evidence : Blob;
  };
  public type ComputeEvidenceArguments = {
    batch_id : BatchId;
    max_iterations : ?Nat16;
  };
  public type ConfigurationResponse = {
    max_batches : ?Nat64;
    max_bytes : ?Nat64;
    max_chunks : ?Nat64;
  };
  public type ConfigureArguments = {
    max_batches : ??Nat64;
    max_bytes : ??Nat64;
    max_chunks : ??Nat64;
  };
  public type CreateAssetArguments = {
    key : Key;
    content_type : Text;
    headers : ?[HeaderField];
    allow_raw_access : ?Bool;
    max_age : ?Nat64;
    enable_aliasing : ?Bool;
  };
  /// Delete an asset
  public type DeleteAssetArguments = { key : Key };
  public type DeleteBatchArguments = { batch_id : BatchId };
  public type GrantPermission = {
    permission : Permission;
    to_principal : Principal;
  };
  public type HeaderField = (Text, Text);
  public type HttpRequest = {
    url : Text;
    method : Text;
    body : Blob;
    headers : [HeaderField];
    certificate_version : ?Nat16;
  };
  public type HttpResponse = {
    body : Blob;
    headers : [HeaderField];
    streaming_strategy : ?StreamingStrategy;
    status_code : Nat16;
  };
  public type InitArgs = { set_permissions : ?SetPermissions };
  public type Key = Text;
  public type ListPermitted = { permission : Permission };
  public type Permission = { #Prepare; #ManagePermissions; #Commit };
  public type RevokePermission = {
    permission : Permission;
    of_principal : Principal;
  };
  /// Add or change content for an asset, by content encoding
  public type SetAssetContentArguments = {
    key : Key;
    sha256 : ?Blob;
    chunk_ids : [ChunkId];
    content_encoding : Text;
  };
  public type SetAssetPropertiesArguments = {
    key : Key;
    headers : ??[HeaderField];
    is_aliased : ??Bool;
    allow_raw_access : ??Bool;
    max_age : ??Nat64;
  };
  /// / Sets the list of principals granted each permission.
  public type SetPermissions = {
    prepare : [Principal];
    commit : [Principal];
    manage_permissions : [Principal];
  };
  public type StreamingCallbackHttpResponse = {
    token : ?StreamingCallbackToken;
    body : Blob;
  };
  public type StreamingCallbackToken = {
    key : Key;
    sha256 : ?Blob;
    index : Nat;
    content_encoding : Text;
  };
  public type StreamingStrategy = {
    #Callback : {
      token : StreamingCallbackToken;
      callback : shared query StreamingCallbackToken -> async ?StreamingCallbackHttpResponse;
    };
  };
  public type Time = Int;
  /// Remove content for an asset, by content encoding
  public type UnsetAssetContentArguments = {
    key : Key;
    content_encoding : Text;
  };
  public type UpgradeArgs = { set_permissions : ?SetPermissions };
  public type ValidationResult = { #Ok : Text; #Err : Text };
  public type Self = ?AssetCanisterArgs -> async actor {
    api_version : shared query () -> async Nat16;
    authorize : shared Principal -> async ();
    certified_tree : shared query {} -> async {
        certificate : Blob;
        tree : Blob;
      };
    clear : shared ClearArguments -> async ();
    /// Perform all operations successfully, or reject
    commit_batch : shared CommitBatchArguments -> async ();
    /// Given a batch already proposed, perform all operations successfully, or reject
    commit_proposed_batch : shared CommitProposedBatchArguments -> async ();
    /// Compute a hash over the CommitBatchArguments.  Call until it returns Some(evidence).
    compute_evidence : shared ComputeEvidenceArguments -> async ?Blob;
    configure : shared ConfigureArguments -> async ();
    create_asset : shared CreateAssetArguments -> async ();
    create_batch : shared {} -> async { batch_id : BatchId };
    create_chunk : shared { content : Blob; batch_id : BatchId } -> async {
        chunk_id : ChunkId;
      };
    create_chunks : shared { content : [Blob]; batch_id : BatchId } -> async {
        chunk_ids : [ChunkId];
      };
    deauthorize : shared Principal -> async ();
    delete_asset : shared DeleteAssetArguments -> async ();
    /// Delete a batch that has been created, or proposed for commit, but not yet committed
    delete_batch : shared DeleteBatchArguments -> async ();
    get : shared query { key : Key; accept_encodings : [Text] } -> async {
        content : Blob;
        sha256 : ?Blob;
        content_type : Text;
        content_encoding : Text;
        total_length : Nat;
      };
    get_asset_properties : shared query Key -> async {
        headers : ?[HeaderField];
        is_aliased : ?Bool;
        allow_raw_access : ?Bool;
        max_age : ?Nat64;
      };
    /// if get() returned chunks > 1, call this to retrieve them.
    /// chunks may or may not be split up at the same boundaries as presented to create_chunk().
    get_chunk : shared query {
        key : Key;
        sha256 : ?Blob;
        index : Nat;
        content_encoding : Text;
      } -> async { content : Blob };
    get_configuration : shared () -> async ConfigurationResponse;
    grant_permission : shared GrantPermission -> async ();
    http_request : shared query HttpRequest -> async HttpResponse;
    http_request_streaming_callback : shared query StreamingCallbackToken -> async ?StreamingCallbackHttpResponse;
    list : shared query {} -> async [
        {
          key : Key;
          encodings : [
            {
              modified : Time;
              sha256 : ?Blob;
              length : Nat;
              content_encoding : Text;
            }
          ];
          content_type : Text;
        }
      ];
    list_authorized : shared () -> async [Principal];
    list_permitted : shared ListPermitted -> async [Principal];
    /// Save the batch operations for later commit
    propose_commit_batch : shared CommitBatchArguments -> async ();
    revoke_permission : shared RevokePermission -> async ();
    set_asset_content : shared SetAssetContentArguments -> async ();
    set_asset_properties : shared SetAssetPropertiesArguments -> async ();
    /// Single call to create an asset with content for a single content encoding that
    /// fits within the message ingress limit.
    store : shared {
        key : Key;
        content : Blob;
        sha256 : ?Blob;
        content_type : Text;
        content_encoding : Text;
      } -> async ();
    take_ownership : shared () -> async ();
    unset_asset_content : shared UnsetAssetContentArguments -> async ();
    validate_commit_proposed_batch : shared CommitProposedBatchArguments -> async ValidationResult;
    validate_configure : shared ConfigureArguments -> async ValidationResult;
    validate_grant_permission : shared GrantPermission -> async ValidationResult;
    validate_revoke_permission : shared RevokePermission -> async ValidationResult;
    validate_take_ownership : shared () -> async ValidationResult;
  }
}
