# To do

## Naive interface
[X] Redesign for multiple public stateful topics / variables
[X] Create GET interface to retrieve stored data
[ ] Create websocket to stream stored data
[X] Factor out storage related code to StorageProvider
[X] Factor out code to HTTP module

## Security
[ ] Choose implementation

### Sign everything
Each client sign its changes. This enables us to selectively include/exclude changes from particular individuals. 
- Implement pod impersonation (client delegates signing) with thin client (just authenticates with pod)
- Implement thin pods (public key authorization for query) with thick client (do their own signing)

#### Questions
- How hard do we need the capability to exclude changes based on their _verifiable_ origin?
  Note: acceptance of some/most? changes would be based on content, rather than origin.
- How valuable is a verifiable audit trail?

### Replicate optimistically, sign for conflict resolution
Changes are still offset on a particular branch. Replication of that branch is transitive.
Inclusion of changes can be done based on content. We replicate a branch optimistically, assuming the data is correct.
When a conflict occurs, a signature from the branch owner can be supplied to establish its authenticity.
 
- Assuming that change-distribution is open/transitive, we can only exclude changes based on content. We may use
origin as a heuristic, but it is not on itself reliable.
- This opens an attack to make parts of the data unavailable. An attacker can replicate incorrect data, requiring the
availability of the origin to sign the correct version and resolve the conflict.
- It opens a window of opportunity. An attacker can replicate incorrect data. Until nodes have received alternate data,
that version will be believed to be true.

This could be sufficient for simple document editing / wiki's. These properties are less acceptable for source code
editing, where an attacker can potentially trigger remote execution of its code. 

## Compression
[ ] Design monotonic rewrites

## Public stateful topics - requirements
- Need to serialize concurrent changes to the same topic