use serde_derive::Deserialize;

#[derive(Debug, Default)]
pub struct ListObjectsRequest<'t> {
    pub prefix: Option<&'t str>,
    pub marker: Option<&'t str>,
    pub max_keys: Option<u32>,
    pub delimiter: Option<&'t str>,
    pub key_marker: Option<&'t str>,
    pub version_id_marker: Option<&'t str>,
}

/*
<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>
<ListBucketResult xmlns=\"http://obs.hc.sbercloud.ru/doc/2015-06-30/\">
    <Name>rust-api-test</Name>
    <Prefix></Prefix>
    <KeyCount>2</KeyCount>
    <MaxKeys>1000</MaxKeys>
    <IsTruncated>false</IsTruncated>
    <Contents>
        <Key>test.txt</Key>
        <LastModified>2023-08-20T11:04:49.119Z</LastModified>
        <ETag>\"bf0b7283b196369ba8723a79750d937b\"</ETag>
        <Size>35</Size>
        <StorageClass>STANDARD</StorageClass>
    </Contents>
    <Contents>
        <Key>test2.txt</Key>
        <LastModified>2023-08-20T11:04:49.250Z</LastModified>
        <ETag>\"bf0b7283b196369ba8723a79750d937b\"</ETag>
        <Size>35</Size>
        <StorageClass>STANDARD</StorageClass>
    </Contents>
</ListBucketResult>
*/
#[derive(Deserialize, Debug)]
pub struct ListObjectsResult {
    #[serde(rename="Name")]
    pub name: String,
    
    #[serde(rename="Prefix")]
    pub prefix: String,
    
    #[serde(rename="KeyCount")]
    pub key_count: Option<u64>,
    
    #[serde(rename="MaxKeys")]
    pub max_keys: Option<u64>,

    #[serde(rename="IsTruncated")]
    pub is_truncated: Option<bool>,

    #[serde(rename="Delimiter")]
    pub delimeter: Option<String>,

    #[serde(rename="Marker")]
    pub marker: Option<String>,

    #[serde(rename="NextMarker")]
    pub next_marker: Option<String>,

    //#[serde(rename="CommonPrefixes")]
    //pub common_prefixes: Option<String>,

    #[serde(rename="Contents")]
    pub contents: Option<Vec<ListObjectsContents>>,
}

#[derive(Deserialize, Debug)]
pub struct ListObjectsContents {
    /// Object name
    #[serde(rename="Key")]
    pub key: String,

    /// Time (UTC) when an object was last modified
    #[serde(rename="LastModified")]
    pub last_modified: String,
    
    /// Base64-encoded 128-bit MD5 digest of an object. ETag is the unique identifier of the object content. 
    /// It can be used to determine whether the object content is changed. 
    /// The actual ETag is the hash value of the object. 
    /// For example, if the ETag value is A when an object is uploaded, but this value has changed to B when 
    /// the object is downloaded, it indicates that the object content has been changed. 
    /// The ETag reflects changes to the object content, rather than the object metadata. 
    /// An uploaded object or copied object has a unique ETag after being encrypted using MD5.
    #[serde(rename="ETag")]
    pub etag: String,
    
    /// Object type. This parameter is returned when the object type is not Normal.
    #[serde(rename="Type")]
    pub type_: Option<String>,
    
    /// Object size in bytes
    #[serde(rename="Size")]
    pub size: u64,

    /// Storage class of an object. Value options: `STANDARD`, `WARM`, `COLD`
    #[serde(rename="StorageClass")]
    pub storage_class: String,

    /// User information, including the domain ID and name of the object owner
    #[serde(rename="Owner")]
    pub owner: Option<Owner>,

    /// Object version ID. Versioned response only
    #[serde(rename="VersionId")]
    pub version_id: Option<String>,

    /// Whether the object is the latest version. If the parameter value is true, the object is the latest version.
    /// Versioned response only
    #[serde(rename="IsLatest")]
    pub is_latest: Option<bool>,
}




#[derive(Deserialize, Debug)]
pub struct Owner {
    /// Domain ID of the object owner
    #[serde(rename="ID")]
    pub id: String,

    /// Name of the object owner
    #[serde(rename="DisplayName")]
    pub display_name: Option<String>,
}

/// Result of list object versions
#[derive(Deserialize, Debug)]
pub struct ListObjectVersionsResult {
    /// Bucket name
    #[serde(rename="Name")]
    pub name: String,
    
    /// Prefix of an object name. Only objects whose names have this prefix are listed. 
    #[serde(rename="Prefix")]
    pub prefix: String,

    /// Marker for the object key from which objects will be listed
    #[serde(rename="KeyMarker")]
    pub key_marker: Option<String>,

    /// Object version ID to start with when objects are listed
    #[serde(rename="VersionIdMarker")]
    pub version_id_marker: Option<String>,

    /// Key marker for the last returned object in the list. 
    /// NextKeyMarker is returned when not all the objects are listed. 
    /// You can set the KeyMarker value to list the remaining objects in follow-up requests.
    #[serde(rename="NextKeyMarker")]
    pub next_key_marker: Option<String>,

    /// Version ID marker for the last returned object in the list. 
    /// NextVersionIdMarker is returned when not all the objects are listed. 
    /// You can set the VersionIdMarker value to list the remaining objects in follow-up requests.
    #[serde(rename="NextVersionIdMarker")]
    pub next_version_id_marker: Option<String>,

    /// Maximum number of objects returned
    #[serde(rename="MaxKeys")]
    pub max_keys: Option<u64>,

    /// Indicates whether the returned list of objects is truncated. 
    /// The value true indicates that the list was truncated and false indicates that the list was not truncated.
    #[serde(rename="IsTruncated")]
    pub is_truncated: Option<bool>,

    /// Container of version information
    #[serde(rename="Version")]
    pub version: Option<Vec<ListObjectsContents>>,
    
    /// Container for objects with deletion markers
    #[serde(rename="DeleteMarker")]
    pub delete_marker: Option<Vec<ListObjectsContents>>,

}


/// Object metadata returned by [obs::Bucket::get_object_meta]
#[derive(Debug)]
pub struct ObjectMeta {
    /// Length of the object in bytes as seen by OBS 
    pub content_length: Option<u64>,
    /// Type of the object's content
    pub content_type: Option<String>,
    /// Date the object was last modified, like "WED, 01 Jul 2015 01:19:21 GMT"
    pub last_modified: Option<String>,

    /// x-obs-expiration
    /// 
    /// When an object has its lifecycle rule, the object expiration time is subject to its lifecycle rule. 
    /// This header field is use expiry-date to describe the object expiration date. If the lifecycle rule is configured 
    /// only for the entire bucket not individual objects, the object expiration time is subject to the bucket lifecycle rule. 
    /// This header field uses the expiry-date and rule-id to describe the detailed expiration information of objects. 
    /// If no lifecycle rule is configured, this header field is not contained in the response.
    /// 
    /// Type: string
    pub expiration: Option<String>,

    /// x-obs-website-redirect-location
    /// 
    /// Indicates the redirected-to location. If the bucket is configured with website information, this parameter can be set 
    /// for the object metadata so that the website endpoint will evaluate the request for the object as a 301 redirect to 
    /// another object in the same bucket or an external URL.
    /// 
    /// Type: string
    pub website_redirect_location: Option<String>,

    /// x-obs-version-id
    /// 
    /// Object version ID. If the object has no version number specified, the response does not contain this header.
    /// 
    /// Type: string
    /// 
    /// Default value: none
    pub version_id: Option<String>,

    /// x-obs-object-type
    /// 
	/// If the object is not a normal one, this header field is returned. The value can be Appendable.
    /// 
    /// Type: string
    pub object_type: Option<String>,

    /// x-obs-next-append-position
    /// 
    /// This header field is returned when the object is an appendable object.
    /// 
    /// Type: integer
    pub next_append_position: Option<u64>,

    /// x-obs-storage-class
    /// 
	/// This header is returned when the storage class of an object is not Standard. The value can be WARM or COLD.
    /// 
    /// Type: string
    pub storage_class: Option<String>,
}


/// Bucket metadata returned by [obs::Bucket::get_bucket_meta]
#[derive(Debug)]
pub struct BucketMeta {
    /// x-obs-bucket-location
    pub bucket_location: Option<String>,
    /// x-obs-storage-class
    pub storage_class: Option<String>,
    /// x-obs-version
    pub version: Option<String>,
    /// x-obs-fs-file-interface
    pub fs_file_interface: Option<String>,
    /// x-obs-epid
    pub epid: Option<String>,
    /// x-obs-az-redundancy
    pub az_redundancy: Option<String>,
}
