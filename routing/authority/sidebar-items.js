initSidebarItems({"enum":[["Authority",""]],"fn":[["our_authority","This returns our calculated authority with regards to the element passed in from the message and the message header. Note that the message has first to pass Sentinel as to be verified. a) if the message is not from a group,       the originating node is within our close group range       and the element is not the destination    -> Client Manager b) if the element is within our close group range       and the destination is the element    -> Network-Addressable-Element Manager c) if the message is from a group,       the destination is within our close group,       and our id is not the destination    -> Node Manager d) if the message is from a group,       the group is within our close group range,       and the destination is our id    -> Managed Node e) otherwise return Unknown Authority"]]});